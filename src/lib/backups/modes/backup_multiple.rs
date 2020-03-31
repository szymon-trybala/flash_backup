use crate::backups::traits::backup::Backup;
use crate::backups::traits::backup_copy::BackupCopy;
use crate::backups::traits::backup_ignore::BackupIgnore;
use crate::backups::traits::backup_input::BackupInput;
use crate::backups::traits::backup_serialize::BackupSerialize;
use crate::backups::traits::backup_output::BackupOutput;
use crate::backups::map::backup_map::BackupMap;
use walkdir::WalkDir;
use crate::{S_MAP, S_SEPARATOR};
use std::fs::{File, create_dir_all};
use std::io::BufReader;
use chrono::Local;
use crate::backups::map::backup_mode::BackupMode;
use std::path::Path;
use crate::backups::helpers::dirs::get_last_subdir;

pub struct BackupMultiple {
    pub map: BackupMap,
    previous_maps: Vec<BackupMap>,
}

impl BackupMultiple {
    pub fn new(map: BackupMap) -> BackupMultiple {
        match map.backup_mode {
            BackupMode::Multiple => {}
            _ => panic!("Mode of created map isn't multiple mode, but multiple mode is trying to be executed")
        }
        let backup_multiple = BackupMultiple { map, previous_maps: vec![] };
        backup_multiple
    }

    /// output_folder has to be filled in "config"
    fn create_new_backup_folder(&mut self) -> Result<(), String> {
        // Checking values
        if self.map.output_folder.is_empty() {
            panic!("Output folder is not set");
        }

        match Path::new(&self.map.output_folder).exists() {
            true => {
                // Finding previous backups
                if let Err(e) = self.find_previous_backups() {
                    let message = format!("Error finding previous backups: {}", e);
                    panic!(message)
                }
                match self.previous_maps.is_empty() {
                    true => println!("No previous maps found"),
                    false => {
                        // Deleting backups until it's less than maximum amount
                        loop {
                            if self.previous_maps.len() >= self.map.max_backups {
                                if let Err(e) = self.delete_oldest_folder() {
                                    let message = format!("Error while removing oldest folder, program will stop: {}", e);
                                    panic!(message);
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
            false => {
                println!("Folder {} doesn't exist, creating...", &self.map.output_folder);
                if let Err(e) = create_dir_all(&self.map.output_folder) {
                    let message = format!("Can't create root outpu folder: {}", e);
                    return Err(message);
                }
            }
        }
        // Creating new backup folder
        let date_now: chrono::DateTime<Local> = Local::now();
        let date_string = date_now.format("%d-%m-%Y %H_%M_%S").to_string();
        let new_backup_path = format!("{}{}{}", self.map.output_folder, S_SEPARATOR, date_string);
        if let Err(e) = create_dir_all(&new_backup_path) {
            let message = format!("Folder {} for new backup can't be created: {}", &new_backup_path, e);
            panic!(message);
        }
        self.map.output_folder = new_backup_path;

        Ok(())
    }

    /// helper to create_new_backup_folder
    fn find_previous_backups(&mut self) -> Result<(), String> {
        self.previous_maps.clear();

        for entry in WalkDir::new(&self.map.output_folder).into_iter().filter_map(|e| e.ok()) {
            if entry.path().ends_with(S_MAP) {
                match File::open(entry.path()) {
                    Err(e) => println!("Error reading possible map, it will not count: {}", e),
                    Ok(previous_map) => {
                        let buf_reader = BufReader::new(previous_map);
                        match serde_json::from_reader(buf_reader) {
                            Ok(previous_map) => self.previous_maps.push(previous_map),
                            Err(e) => println!("Possible map, not valid, it will not count: {}", e)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Use after "find_previous_backups", helper to create_new_backup_folder
    fn delete_oldest_folder(&mut self) -> Result<(), String> {
        // Checking if value exists
        if self.previous_maps.is_empty() {
            return Err(String::from("No previous maps found"));
        }

        // Finding oldest backup
        let mut oldest_map_timestamp: usize = std::usize::MAX;
        let mut oldest_map_output_path = String::new();

        for previous_map in &self.previous_maps {
            if previous_map.timestamp < oldest_map_timestamp {
                oldest_map_timestamp = previous_map.timestamp;
                oldest_map_output_path = previous_map.output_folder.clone();
            }
        }

        println!("Max amount of backups reached, deleting oldest one: {}", &oldest_map_output_path);
        match std::fs::remove_dir_all(&oldest_map_output_path) {
            Ok(_) => {
                if let Err(e) = self.find_previous_backups() {
                    println!("Error: deleted folder is still on list, program thinks there are {} backups", self.previous_maps.len());
                    return Err(e);
                }
                Ok(())
            },
            Err(e) => {
                let message = format!("Error removing oldest directory {}: {}. Program will stop", &oldest_map_output_path, e);
                panic!(message);
            }
        }
    }
}

impl BackupCopy for BackupMultiple {}

impl BackupIgnore for BackupMultiple {}

impl BackupInput for BackupMultiple {}

impl BackupSerialize for BackupMultiple {}

impl Backup for BackupMultiple {
    fn backup(&mut self) -> Result<(), String> {
        if let Err(e) = self.create_new_backup_folder() {
            let message = format!("Couldn't create new backup folder: {}. Program will stop", e);
            panic!(message);
        }
        self.map.backup_dirs = BackupMultiple::create_input_maps(&self.map.input_folders);
        let mut copied = self.map.clone();
        copied.backup_dirs = BackupMultiple::ignore_files_and_folders_parrarel(copied.backup_dirs, &copied.ignore_extensions, &copied.ignore_folders);
        let mut copied = copied.clone();
        let mut copied = BackupMultiple::create_output_map(copied);
        copied.backup_dirs = BackupMultiple::copy_all(copied.backup_dirs);
        BackupMultiple::serialize_to_json(&mut copied);

        // self.map = copied.clone();
        Ok(())
    }
}

impl BackupOutput for BackupMultiple {
    fn create_output_map(mut map: BackupMap) -> BackupMap {
        // Checking if create_backup_folder has been executed
        if map.output_folder.is_empty() {
            println!("Root output folder isn't set up");
            return map;
        }
        // Creating output paths
        for dir in &mut map.backup_dirs {
            if dir.root_input.is_empty() {
                println!("At least one of main input folders isn't set up");
                return map;
            }
            match get_last_subdir(&dir.root_input) {
                Err(e) => println!("Can't create output folder to backup {}: {}, skipping...", &dir.root_input, e),
                Ok(last_subdir) => {
                    dir.root_output = format!("{}{}{}", &map.output_folder, S_SEPARATOR, last_subdir);
                    for entry in &mut dir.backup_entries {
                        if entry.input_path.is_empty() {
                            println!("At least one entry don't have filled input path");
                            return map;
                        }
                        entry.output_path = entry.input_path.replacen(&dir.root_input, &dir.root_output, 1);
                    }
                }
            }
        }
        map
    }
}

