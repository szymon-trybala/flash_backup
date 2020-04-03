use crate::backups::traits::backup_input::BackupInput;
use crate::backups::traits::backup_ignore::BackupIgnore;
use crate::backups::map::backup_map::BackupMap;
use crate::backups::traits::backup_copy::BackupCopy;
use crate::backups::map::backup_mode::BackupMode;
use crate::backups::traits::backup_serialize::BackupSerialize;
use crate::backups::map::backup_dir::BackupDir;
use scoped_threadpool::Pool;
use std::sync::{Arc, Mutex};
use std::borrow::{Borrow, BorrowMut};
use crate::backups::helpers::multithreading::arc_to_inner;
use std::fs::{remove_file, create_dir_all};
use crate::backups::traits::backup_output::BackupOutput;
use crate::backups::helpers::dirs::{get_last_subdir, find_previous_backups, delete_folder_with_content};
use crate::S_SEPARATOR;
use crate::backups::traits::backup::Backup;
use std::path::Path;

pub struct BackupCloud {
    pub map: BackupMap,
    pub previous_map: BackupMap,
    pub copy_dirs: Vec<BackupDir>,
    matching_dirs: (Vec<(usize, usize)>, Vec<usize>),
}

impl BackupCloud {
    /// Creates new BackupCloud struct, requires already created BackupMap with filled basic informations like input folders and output folder.
    ///
    /// May panic if data have not been filled, or if map's mode isn't multiple.
    pub fn new(map: BackupMap) -> BackupCloud {
        if map.output_folder.is_empty() || map.input_folders.is_empty() {
            panic!("Not all needed data filled. Program will stop");
        }
        match map.backup_mode {
            BackupMode::Cloud => {}
            _ => panic!("Mode of created map isn't in cloud mode, but cloud mode is trying to be executed. Program will stop")
        }
        let backup_cloud = BackupCloud { map, previous_map: BackupMap::new(BackupMode::Cloud), copy_dirs: vec![], matching_dirs: (vec![], vec![])};
        backup_cloud
    }

    /// Searches for new or modified entries in folders and puts them in ```copy_dirs``` fields.
    ///
    /// Works in multi threads (max amount of them is equal to computer's thread count).
    ///
    /// New or modified file is file with different hash then it's equivalent in backup.
    ///
    /// Shoud be used after filling input and ignoring, alongside with deleting missing files.
    ///
    /// Returns error if there isn't any previously created backup.
    /// Function can panic if fatal error occurs during multithreading operations and conversions - it's too dangerous to continue runtime at this point. Function returns error if provided map is empty.
    /// Minor errors are printed and do not stop execution of program.
    pub fn generate_entries_to_copy_all(&mut self) -> Result<(), String> {
        // Checking input
        if self.map.backup_dirs.is_empty() {
            return Err(String::from("Map of files is empty!"));
        }
        println!("Looking for new or modified files and folders...");

        // Creating needed variables
        let copy_dirs = Arc::new(Mutex::new(vec![]));
        let dirs = Arc::new(&self.map.backup_dirs);
        let previous_dirs = Arc::new(&self.previous_map.backup_dirs);
        let max_threads = num_cpus::get();
        let mut thread_pool = Pool::new(max_threads as u32);
        let matching_dirs = self.find_linked_dirs(&self.map.backup_dirs, &self.previous_map.backup_dirs);
        self.matching_dirs = matching_dirs.clone();
        let matching_dirs = Arc::new(matching_dirs);
        // Searching matching

        // Creating threads and ignoring files from all of them
        thread_pool.scoped(|scope| {
            // Handling dirs that doesn't exist in current backup
            if matching_dirs.1.len() > 0 {
                for dir_without_match_index in &matching_dirs.1 {
                    println!("{} not found in existing backup, will copy all its content", &self.map.backup_dirs[*dir_without_match_index].root_input);
                    let mut copy_dirs_temp = copy_dirs.lock().unwrap();
                    copy_dirs_temp.push(self.map.backup_dirs[*dir_without_match_index].clone());
                }

            }

            // Handling dirs that exist in current backup, but may be different
            let len = matching_dirs.0.len();
            for i in 0..len {
                // Mutable references to all needed values
                let dirs_ref = Arc::clone(&dirs);
                let previous_dirs_ref = Arc::clone(&previous_dirs);
                let matching_dirs_ref = Arc::clone(&matching_dirs);
                let copy_dirs_ref = Arc::clone(&copy_dirs);
                scope.execute(move || {
                    // Adding folder with new/modified entries to copy_dirs
                    let backup_folder = generate_entries_to_copy_one_folder(dirs_ref[matching_dirs_ref.0[i].0].borrow(), previous_dirs_ref[matching_dirs_ref.0[i].1].borrow());
                    if backup_folder.files > 0 {
                        let mut copy_dirs_temp = copy_dirs_ref.lock().unwrap();
                        copy_dirs_temp.push(backup_folder);
                    }
                });
            }
        });
        match arc_to_inner(copy_dirs) {
            Ok(copy_dirs) => {
                self.copy_dirs = copy_dirs;
                Ok(())
            },
            Err(e) => {
                let message = format!("Fatal error while trying to find new or modified files - {}. Program will stop", e);
                panic!(message);
            }
        }
    }

    /// Finds the same ```BackupDir```'s in already created backup and newly checked folders - dirs are "the same" if their root inputs are equal.
    fn find_linked_dirs(&self, dirs: &Vec<BackupDir>, previous_dirs: &Vec<BackupDir>) -> (Vec<(usize, usize)>, Vec<usize>) {
        let mut matching = vec![];
        let mut without_match = vec![];
        for (dir_index, dir) in dirs.iter().enumerate() {
            let mut found_matching = false;
            for (previous_dir_index, previous_dir) in previous_dirs.iter().enumerate() {
                if previous_dir.root_input == dir.root_input {
                    found_matching = true;
                    matching.push((dir_index, previous_dir_index));
                    break;
                }
            }
            if !found_matching {
                without_match.push(dir_index);
            }
        }

        (matching, without_match)
    }

    /// Deletes from backup folders all files and folders that doesn't exist in latest version of input folders.
    ///
    /// Shoud be used after filling input and ignoring, alongside with copying new/modified entries.
    ///
    /// Works in multi threads (max amount of them fixed at 4, to not overload the hard disk).
    ///
    /// Returns error if there isn't any previously created backup, or if there are no matching folders.
    /// Function can panic if fatal error occurs during multithreading operations and conversions - it's too dangerous to continue runtime at this point. Function returns error if provided map is empty.
    /// Minor errors are printed and do not stop execution of program.
    pub fn delete_missing_all(&self) -> Result<(), String> {
        println!("Looking for deleted folders and files...");
        // Checking input
        if self.previous_map.backup_dirs.is_empty() {
            return Err(String::from("Can't find dirs from previous backup"));
        }
        if self.matching_dirs.0.is_empty() {
            return Err(String::from("No matching dirs to remove redundant files"));
        }

        // Creating needed variables
        let dirs = Arc::new(&self.map.backup_dirs);

        let previous_dirs = self.previous_map.backup_dirs.clone();
        let previous_dirs = Arc::new(Mutex::new(previous_dirs));

        let mut thread_pool = Pool::new(4);
        let matching_dirs = Arc::new(&self.matching_dirs.0);
        let len = self.matching_dirs.0.len();

        // Removing redundant entries from folders, one folder per thread
        thread_pool.scoped(|scope| {
            for i in 0..len {
                // Mutable references to all needed values
                let dirs_ref = Arc::clone(&dirs);
                let previous_dirs_ref = Arc::clone(&previous_dirs);
                let matching_dirs_ref = Arc::clone(&matching_dirs);
                scope.execute(move || {
                    // Removing redundant files/folders from backup inputs
                    let mut previous_dirs_temp = previous_dirs_ref.lock().unwrap();
                    match delete_missing_one_folder(dirs_ref[matching_dirs_ref[i].0].borrow(), previous_dirs_temp[matching_dirs_ref[i].1].borrow_mut()) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                })
            }
        });
        Ok(())
    }
}

/// Searches for new or modified entries in one folder.
///
/// Compares folders content with its equivalent in backup and returns ```BackupDir``` of those entries that doesn't have equivalent in backup.
///
///
/// New or modified file is file with different hash then it's equivalent in backup.
fn generate_entries_to_copy_one_folder(folder: &BackupDir, previous_folder: &BackupDir) -> BackupDir {
    let mut counter: usize = 0;
    let mut copy_folder = BackupDir::new();
    if previous_folder.backup_entries.is_empty() {
        copy_folder.backup_entries = folder.backup_entries.clone();
    }
    for entry in &folder.backup_entries {
        if entry.is_file {
            // Checking if folder contains the same file
            let mut found_matching_hash = false;
            for previous_entry in &previous_folder.backup_entries {
                if previous_entry.hash == entry.hash {
                    found_matching_hash = true;
                    break;
                }
            }

            // Adding new/modified file to copy_folder
            if !found_matching_hash {
                counter += 1;
                copy_folder.backup_entries.push(entry.clone());
            }
        }
    }
    match counter {
        0 => println!("No new files found in {}", &previous_folder.root_output),
        _ => println!("{} new or modified files found in {}", counter, &previous_folder.root_input),
    }

    // Copying metadata
    copy_folder.root_input = previous_folder.root_input.clone();
    copy_folder.root_output = previous_folder.root_output.clone();
    copy_folder.files = copy_folder.backup_entries.iter().filter(|x| x.is_file).count();    // copy_folder contains only files, so it makes no sense to count folders

    copy_folder
}

/// Deletes from folder all files and folders that doesn't exist in latest version of input folders, deletes them also from map.
///
/// The function acknowledges that the file does not exist in current version of folder if no file in current version of folder has the same hash.
///
/// Function returns error if folder can't be deleted recursively.
fn delete_missing_one_folder(folder: &BackupDir, previous_folder: &mut BackupDir) -> Result<(), String> {
    let mut deleted: usize = 0;

    for previous_entry in &previous_folder.backup_entries {
        let mut found = false;
        for entry in &folder.backup_entries {
            if previous_entry.hash == entry.hash {
                found = true;
                break;
            }
        }
        if !found {
            if previous_entry.is_file {
                if let Err(_) = remove_file(&previous_entry.output_path) {
                    continue;
                }
                deleted += 1;
            } else {
                match delete_folder_with_content(&previous_entry.output_path, &folder) {
                    Ok(removed) => deleted += removed,
                    Err(e) => {
                        let message = format!("Can't remove folder {} with its content: {}", &previous_entry.output_path, e);
                        return Err(message);
                    }
                }
            }
        }
    }
    match deleted {
        0 => println!("No deleted files in {} found", &folder.root_input),
        _ => println!("Deleted {} redundant files from {}", deleted, &folder.root_input)
    }

    Ok(())
}

impl BackupCopy for BackupCloud {}

impl BackupIgnore for BackupCloud {}

impl BackupInput for BackupCloud {}

impl BackupSerialize for BackupCloud {}

impl BackupOutput for BackupCloud {
    /// Creates output map - for each entry, output folder is changed to one based on root output folder.
    ///
    /// May panic if root output folder is empty, prints to user any other possible, application non-breaking error.
    fn create_output_map(mut map: BackupMap) -> BackupMap {
        // Checking if create_backup_folder has been executed
        if map.output_folder.is_empty() {
            panic!("Root output folder isn't set up. Program will stop");
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
                    dir.root_output = format!("{}{}{}", map.output_folder, S_SEPARATOR, last_subdir);
                    for entry in &mut dir.backup_entries {
                        entry.output_path = entry.input_path.replacen(&dir.root_input, &dir.root_output, 1);
                    }
                }
            }
        }
        map
    }
}

impl Backup for BackupCloud {
    /// Main function of BackupCloud, that runs all corresponding functions.
    ///
    /// First it checks if output folder exists. If yes, it tries to find previous backup. If no, folder is created.
    /// Then map is filled with data and ignoring is applied. Then, program checks new or modified files and saves them to separate field, redundant files and folders are also removed from backup.
    /// Then if any files to copy are present, it copies all of them and checks integrity of both files and map.
    ///
    /// Function may panic if required variables are empty, or if functions in traits panic. Every non-panic error is printed to user.
    fn backup(&mut self) -> Result<(), String> {
        if self.map.output_folder.is_empty() || self.map.input_folders.is_empty() {
            panic!("Trying to backup in cloud mode, but basic metadata is not filled. Program will stop");
        }
        match Path::new(&self.map.output_folder).exists() {
            true => {
                match find_previous_backups(&self.map.output_folder) {
                    Err(e) => {
                        let message = format!("Couldn't find previous backups: {}. Program will stop", e);
                        panic!(message);
                    }
                    Ok(backups) => {
                        if backups.len() > 1 {
                            panic!(String::from("Found too many backups, program will stop"));
                        }
                        self.previous_map = backups[0].clone();
                    }
                }
            },
            false => {
                if let Err(_) = create_dir_all(&self.map.output_folder) {
                    panic!("Can't create output folder. Program will stop")
                }
            }
        }

        // Filling map with data - find better way without moving so much data
        self.map.backup_dirs = BackupCloud::create_input_maps(&self.map.input_folders);
        let map_copy = self.map.clone();
        self.map.backup_dirs = BackupCloud::ignore_files_and_folders_parrarel(map_copy.backup_dirs, &map_copy.ignore_extensions, &map_copy.ignore_folders);
        let map_copy = self.map.clone();
        self.map = BackupCloud::create_output_map(map_copy);

        // Filling copy_dirs and deleting redundant files
        if let Err(e) = self.generate_entries_to_copy_all() {
            let message = format!("Couldn't generate new/modified entries to copy: {}", e);
            panic!(message);
        }
        if !self.previous_map.backup_dirs.is_empty() {
            if let Err(e) = self.delete_missing_all() {
                println!("Couldn't delete redundant entries: {}", e);
            }
        }

        // Copying and verifying data
        match self.copy_dirs.is_empty() {
            true => {
                println!("No need to copy any files, program will stop now");
            }
            false => {
                // Deleting non-confirmed entries from map
                self.copy_dirs = BackupCloud::copy_all(self.copy_dirs.clone());
                let copied = self.map.backup_dirs.clone();
                self.map.backup_dirs = BackupCloud::delete_non_existing(copied);

                // Serializing map
                if let Err(e) = BackupCloud::serialize_to_json(&mut self.map) {
                    println!("Map couldn't be saved to file, this backup won't be considered next time: {}", e);
                }
                // Verifying files
                if let Err(e) = BackupCloud::verify_all(&self.map) {
                    println!("Error while verifying integrity of copied files: {}", e);
                }
            }
        }
        Ok(())
    }
}
