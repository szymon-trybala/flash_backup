use std::path::{Path, MAIN_SEPARATOR};
use std::{fs, io};
use crate::FILE_MAP_NAME;
use std::error::Error;
use crate::io::serialization::{Serialization, Entry};
use walkdir::WalkDir;
use crate::modes::Mode;
use std::io::{BufReader, BufWriter};
use std::convert::TryFrom;

pub struct Cloud {
    pub backup: Serialization,
    pub source: Serialization,
    pub compared: Serialization,
}

impl Cloud {
    pub fn new() -> Cloud {
        Cloud { backup: Serialization::new(), source: Serialization::new(), compared: Serialization::new()}
    }

    pub fn load_existing_serialization(&mut self, folder: &Path) -> Result<(), Box<dyn Error>> {
        if !(folder.exists() && folder.is_dir()) {
            fs::create_dir_all(folder)?;
        }

        // LOADING SERIALIZATION FROM FILE
        let map_path = String::from(folder.to_str().unwrap()) + MAIN_SEPARATOR.to_string().as_ref() + FILE_MAP_NAME;
        let file = fs::File::open(map_path)?;
        let buf_reader = io::BufReader::new(file);
        let map: Serialization = serde_json::from_reader(buf_reader)?;

        self.backup = map;
        self.compared.metadata.output_folder = String::from(folder.to_str().unwrap());
        Ok(())
    }

    pub fn create_new_serialization(&mut self, input_path: &Path) -> Result<(), &'static str> {
        if !(input_path.exists() && input_path.is_dir()) {
            return Err("invalid path of input folder");
        }

        // CREATING MAP OF ALL INPUT FOLDERS AND FILES
        let mut entries = Vec::new();
        for copied_entry in WalkDir::new(input_path).into_iter().filter_map(|e| e.ok()) {
            entries.push(copied_entry);
        }

        // CREATING COMPLETE SERIALIZATION STRUCT
        let mut serialization = Serialization::new();
        serialization.generate_map(&entries);
        serialization.generate_metadata(&vec![String::from(input_path.to_str().unwrap())], input_path.to_str().unwrap(), &Mode::Cloud);

        self.source = serialization;
        self.compared.metadata.input_folders.push(input_path.to_str().unwrap().to_string());
        Ok(())
    }

    pub fn generate_entries_to_copy(&mut self) -> Result<(), &'static str> {
        // TODO - TERRIBLE PERFORMANCE, FIX IT
        if self.source.metadata.mode != Mode::Cloud {
            return Err("at least one of maps isn't in cloud mode");
        }
        if self.source.map.is_empty() {
            return Err("no entries to copy");
        }

        let mut entries_to_copy = Vec::new();
        let mut counter: usize = 0;
        for entry in &self.source.map {
            if entry.is_file {
                let mut found_matching_hash = false;
                for old_entry in &self.backup.map {
                    if entry.hash == old_entry.hash {
                        found_matching_hash = true;
                        break;
                    }
                }
                if !found_matching_hash {
                    entries_to_copy.push(entry.clone());
                    counter += 1;
                }
            } else {
                let mut folder_exists = false;
                for old_entry in &self.backup.map {
                    if !old_entry.is_file && old_entry.path == entry.path {
                        folder_exists = true;
                    }
                }
                if !folder_exists {
                    entries_to_copy.push(entry.clone());
                }
            }

        }

        match entries_to_copy.is_empty() {
            true => Err("no new files or folders"),
            false => {
                println!("Detected {} entries to copy", counter);
                self.compared.map = entries_to_copy;
                Ok(())
            }
        }
    }

    pub fn delete_missing(&mut self) -> Result<(), Box<dyn Error>> {
        let mut deleted: usize = 0;
        if self.backup.map.is_empty() {
            return Err(Box::try_from("backup folder is empty, no files to delete!").unwrap())
        }
        for backup_entry in &self.backup.map {
            let backup_entry_with_changed_root = backup_entry.path.replacen(&self.backup.metadata.output_folder[..], &self.source.metadata.output_folder[..], 1);
            let mut found = false;

            for current_entry in &self.source.map {
                if current_entry.path == backup_entry_with_changed_root {
                    found = true;
                    break;
                }
            }

            if !found {
                if backup_entry.is_file {
                    if let Err(_) = fs::remove_file(&backup_entry.path) {
                        continue;
                    }
                    deleted += 1;
                } else {
                    // DELETING FOLDER AND ITS CONTENT
                    match self.delete_folder_with_content(&self.backup.map, &backup_entry.path) {
                        Ok(x) => {
                            deleted += x;
                        }
                        Err(e) => {
                            println!("Error while deleting folder {}: {}", &backup_entry.path, e);
                            continue;
                        }
                    }
                }
            }
        }
        println!("Deleted {} redundant entries", deleted);
        Ok(())
    }

    fn delete_folder_with_content(&self, map: &Vec<Entry>, path: &str) -> Result<usize, Box<dyn Error>> {
        let mut entries: usize = 0;
        let path_with_separator = String::from(path) + MAIN_SEPARATOR.to_string().as_str();
        let folder_content: Vec<Entry> = map.iter().filter(|x| x.path.starts_with(&path_with_separator)).cloned().collect();
        if folder_content.is_empty() {
            fs::remove_dir(path)?;
            entries += 1;
            return Ok(entries);
        }
        for item in &folder_content {
            if item.is_file {
                fs::remove_file(&item.path)?;
                entries += 1;
            } else {
                if let Err(e) = self.delete_folder_with_content(&folder_content, &item.path) {
                   return Err(e);
                }
                fs::remove_dir(&item.path)?;
                entries += 1;
            }
        }
        fs::remove_dir(&path)?;
        entries += 1;
        Ok(entries)
    }

    pub fn skip_root_paths(&mut self) {
        if !self.source.map.is_empty() && self.source.map[0].path == self.source.metadata.output_folder {
            self.source.map.remove(0);
        }

        if !self.backup.map.is_empty() && self.backup.map[0].path == self.backup.metadata.output_folder {
            self.backup.map.remove(0);
        }

        if !self.compared.map.is_empty() && self.compared.map[0].path == self.source.metadata.output_folder {
            self.compared.map.remove(0);
        }
    }

    pub fn copy_compared(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO - SKIPPING ROOT OUTPUT FOLDER SHOULD BE DONE BEFORE, FIX IT
        let mut file_counter: usize = 0;

        let root_destination = self.backup.metadata.output_folder.clone();
        let root_source = self.source.metadata.output_folder.clone();

        for current_source in &self.compared.map {
            let current_destination = current_source.path.replacen(root_source.as_str(), &root_destination[..], 1);

            // CREATE FOLDER
            if !current_source.is_file {
                fs::create_dir_all(&current_destination)?;
            }

            // CREATE FILE
            if current_source.is_file {
                let source_file = fs::File::open(&current_source.path)?;
                let mut reader = BufReader::new(source_file);

                let destination_file = fs::File::create(&current_destination)?;
                let mut writer = BufWriter::new(destination_file);

                io::copy(&mut reader, &mut writer)?;
                file_counter += 1;
            }
        }
        println!("Copied {} files", file_counter);
        Ok(())
    }

    pub fn save_json(&mut self) -> Result<(), String> {
        // TODO - TERRIBLE PERFORMANCE, UNNECESSERY COPYING, FIX IT
        // Creating new map of folder after all changes
        let mut entries = Vec::new();
        for copied_entry in WalkDir::new(&self.backup.metadata.output_folder).into_iter().filter_map(|e| e.ok()) {
            entries.push(copied_entry);
        }
        let mut serialization = Serialization::new();
        if let Err(e) = serialization.generate_map(&entries) {
            let message = String::from("couldn't generate map: ") + e;
            return Err(message);
        }
        serialization.generate_metadata(&self.backup.metadata.input_folders, &self.backup.metadata.output_folder, &Mode::Cloud);
        if let Err(e) = serialization.serialize_to_json(&self.backup.metadata.output_folder[..]) {
            let message = String::from("couldn't serialize to JSON: ") + e;
            return Err(message);
        }

        Ok(())
    }
}