use std::path::{Path, MAIN_SEPARATOR};
use std::{fs, io};
use crate::FILE_MAP_NAME;
use std::error::Error;
use crate::io::serialization::{Serialization, Entry};
use walkdir::WalkDir;
use crate::modes::Mode;
use std::io::{BufReader, BufWriter};
use std::convert::TryFrom;
use std::collections::HashMap;

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

    pub fn create_new_serialization(&mut self, input_paths: &Vec<String>, output_path: &str) -> Result<(), &'static str> {
        if input_paths.is_empty() {
            return Err("no files to copy");
        }
        self.source.metadata.output_folder = self.backup.metadata.output_folder.clone();

        for path in input_paths {
            let as_path = Path::new(path);
            if !(as_path.exists() && as_path.is_dir()) {
                return Err("invalid path of input folder");
            }

            // CREATING MAP OF ENTRIES IN FOLDER
            let mut entries = Vec::new();
            for copied_entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                entries.push(copied_entry);
            }

            // CREATING COMPLETE SERIALIZATION STRUCT
            self.source.generate_map(&path[..], &entries);
        }
        self.source.generate_metadata(output_path, &Mode::Cloud);
        Ok(())
    }

    pub fn generate_entries_to_copy(&mut self) -> Result<(), &'static str> {
        let mut counter: usize = 0;
        if self.source.metadata.mode != Mode::Cloud {
            return Err("map of input files is empty");
        }
        if self.source.maps.is_empty() {
            return Err("no entries to copy");
        }

        let mut entries_to_copy = HashMap::new();


        for (root, entries) in &self.source.maps {
            match self.backup.maps.get_key_value(root) {
                Some((backup_key, backup_entries)) => {
                    // This folder exists in backup, checking if there are new files
                    let mut new_entries = Vec::new();
                    for input_entry in entries {
                        if input_entry.is_file {
                            let mut found_matching_hash = false;
                            for backup_entry in backup_entries {
                                if input_entry.hash == backup_entry.hash {
                                    found_matching_hash = true;
                                    break;
                                }
                            }
                            if !found_matching_hash {
                                counter += 1;
                                new_entries.push(input_entry.clone());
                            }
                        }
                        else {
                            let mut folder_exists = false;
                            for backup_entry in backup_entries {
                                if !backup_entry.is_file && backup_entry.path == input_entry.path {
                                    folder_exists = true;
                                    break;
                                }
                            }
                            if !folder_exists {
                                new_entries.push(input_entry.clone());
                            }
                        }
                    }
                    entries_to_copy.insert(root.clone(), new_entries);
                }
                None => {
                    // If this folder doesn't exist in backup, algorithm will copy all its content.
                    entries_to_copy.insert(root.clone(), entries.clone());
                }
            }
        }

        match entries_to_copy.is_empty() {
            true => Err("no new files or folders"),
            false => {
                println!("Detected {} entries to copy", counter);
                self.compared.maps = entries_to_copy;
                self.compared.generate_metadata(&self.source.metadata.output_folder, &Mode::Cloud);
                Ok(())
            }
        }
    }

    pub fn delete_missing(&mut self) -> Result<(), Box<dyn Error>> {
        let mut deleted: usize = 0;
        if self.backup.maps.is_empty() {
            return Err(Box::try_from("backup folder is empty, no files to delete!").unwrap())
        }

        for (backup_entries_input, backup_entries) in &self.backup.maps {
            match self.source.maps.get_key_value(backup_entries_input) {
                Some((source_root, source_entries)) => {
                    for backup_entry in backup_entries {
                        let mut found = false;
                        for source_entry in source_entries {
                            if backup_entry.path == source_entry.path {
                                found = true;
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                        else {
                            if backup_entry.is_file {
                                if let Err(_) = fs::remove_file(&backup_entry.path) {
                                    continue;
                                }
                                deleted += 1;
                            } else {
                                // DELETING FOLDER AND ITS CONTENT
                                match self.delete_folder_with_content(&backup_entries, &backup_entries_input) {
                                    Ok(x) => {
                                        deleted += x;
                                    }
                                    Err(e) => {
                                        println!("Error while deleting folder {}: {}", &backup_entries_input, e);
                                        continue;
                                    }
                                }
                            }
                        }

                    }
                }
                None => {
                    // If this folder doesn't exist in source, algorithm will delete it from backup
                    match self.delete_folder_with_content(&backup_entries, &backup_entries_input) {
                        Ok(x) => {
                            deleted += x;
                        }
                        Err(e) => {
                            println!("Error while deleting folder {}: {}", &backup_entries_input, e);
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

    // pub fn skip_root_paths(&mut self) {
    //     if !self.source.map.is_empty() && self.source.map[0].path == self.source.metadata.output_folder {
    //         self.source.map.remove(0);
    //     }
    //
    //     if !self.backup.map.is_empty() && self.backup.map[0].path == self.backup.metadata.output_folder {
    //         self.backup.map.remove(0);
    //     }
    //
    //     if !self.compared.map.is_empty() && self.compared.map[0].path == self.source.metadata.output_folder {
    //         self.compared.map.remove(0);
    //     }
    // }

    pub fn copy_compared(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO - SKIPPING ROOT OUTPUT FOLDER SHOULD BE DONE BEFORE, FIX IT
        let mut file_counter: usize = 0;
        let root_destination = self.backup.metadata.output_folder.clone();

        for (compared_root, comparted_entries) in &self.compared.maps {
            let folder_source = compared_root.clone();
            let folder_source_splitted: Vec<&str> = folder_source.split(MAIN_SEPARATOR).collect();
            let relative_root;
            match folder_source_splitted.last() {
                Some(last) => {
                    relative_root = last;
                }
                None => {
                    println!("Error while converting folder paths in {}, skipping...", compared_root);
                    continue;
                }
            }
            for compared_entry in comparted_entries {
                let combined = root_destination.clone() + MAIN_SEPARATOR.to_string().as_str() + *relative_root;
                let current_destination = compared_entry.path.replacen(folder_source.as_str(), combined.as_str(), 1);

                // CREATE FOLDER
                if !compared_entry.is_file {
                    fs::create_dir_all(&current_destination)?;
                } else {
                    // CREATE FILE
                    let source_file = fs::File::open(&compared_entry.path)?;
                    let mut reader = BufReader::new(source_file);

                    let destination_file = fs::File::create(&current_destination)?;
                    let mut writer = BufWriter::new(destination_file);

                    io::copy(&mut reader, &mut writer)?;
                    file_counter += 1;
                }
            }

        }
        println!("Copied {} files", file_counter);
        Ok(())
    }

    pub fn save_json(&mut self) -> Result<(), String> {
        let mut serialization = Serialization::new();
        serialization.maps = self.source.maps.clone();
        let output = self.backup.metadata.output_folder.clone();
        serialization.replace_in_paths(&output);

        serialization.generate_metadata(&self.backup.metadata.output_folder, &Mode::Cloud);
        if let Err(e) = serialization.serialize_to_json(&self.backup.metadata.output_folder[..]) {
            let message = String::from("couldn't serialize to JSON: ") + e;
            return Err(message);
        }
        Ok(())
    }
}