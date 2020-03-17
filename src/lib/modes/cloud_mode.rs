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
    pub copying_error_paths: Vec<String>,
}

impl Cloud {
    pub fn new() -> Cloud {
        Cloud { backup: Serialization::new(), source: Serialization::new(), compared: Serialization::new(), copying_error_paths: Vec::new()}
    }

    pub fn load_existing_serialization(&mut self, folder: &Path) -> Result<(), Box<dyn Error>> {
        if !(folder.exists() && folder.is_dir()) {
            fs::create_dir_all(folder)?;
        }

        // LOADING SERIALIZATION FROM FILE
        let map_path = String::from(folder.to_str().unwrap()) + MAIN_SEPARATOR.to_string().as_ref() + FILE_MAP_NAME;
        println!("Looking for existing map in folder {}", &map_path[..]);
        let file = fs::File::open(map_path)?;
        let buf_reader = io::BufReader::new(file);
        let map: Serialization = serde_json::from_reader(buf_reader)?;

        self.backup = map;
        self.compared.metadata.output_folder = String::from(folder.to_str().unwrap());
        println!("Found file map!");
        Ok(())
    }

    pub fn create_new_serialization(&mut self, input_paths: &Vec<String>, output_path: &str) -> Result<(), &'static str> {
        println!("Creating maps of desired folders...");
        if input_paths.is_empty() {
            return Err("no files to copy");
        }
        self.source.metadata.output_folder = self.backup.metadata.output_folder.clone();

        for path in input_paths {
            println!("Creating map of {}...", path);
            let as_path = Path::new(path);
            if !(as_path.exists() && as_path.is_dir()) {
                return Err("invalid path of input folder");
            }

            // CREATING MAP OF ENTRIES IN FOLDER
            let mut entries = Vec::new();
            for copied_entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                entries.push(copied_entry);
            }
            if entries.len() <= 1 {
                return Err("one of input folders is empty");
            }

            // CREATING COMPLETE SERIALIZATION STRUCT
            if let Err(e) = self.source.generate_map(&path[..], &entries) {
                println!("Fatal error while creating file map: {}", e);
                panic!();
            }
            println!("Map created!")
        }
        self.source.generate_metadata(output_path, &Mode::Cloud);
        Ok(())
    }

    pub fn generate_entries_to_copy(&mut self) -> Result<(), &'static str> {
        println!("Looking for new or modified files and folders...");
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
                Some((_, backup_entries)) => {
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
                        else if !input_entry.is_file {
                            let mut folder_exists = false;
                            for backup_entry in backup_entries {
                                let root_splitted: Vec<&str> = root.split(MAIN_SEPARATOR).collect();
                                let last = root_splitted.last();
                                let output_folder_with_last;
                                match last {
                                    Some(last) => output_folder_with_last = String::from(&self.backup.metadata.output_folder) + MAIN_SEPARATOR.to_string().as_str() + last,
                                    None => output_folder_with_last = String::from(&self.backup.metadata.output_folder)
                                }
                                let backup_entry_to_compare = backup_entry.path.replacen(&output_folder_with_last, root.as_str(), 1);
                                if backup_entry_to_compare == input_entry.path {
                                    folder_exists = true;
                                    break;
                                }
                            }
                            if !folder_exists {
                                new_entries.push(input_entry.clone());
                            }
                        }
                    }
                    if new_entries.len() > 0 {
                        entries_to_copy.insert(root.clone(), new_entries);
                    }
                }
                None => {
                    // If this folder doesn't exist in backup, algorithm will copy all its content.
                    entries_to_copy.insert(root.clone(), entries.clone());
                    counter += entries.into_iter().filter(|x| x.is_file).count();
                }
            }
        }

        match entries_to_copy.is_empty() {
            true => Err("No new files or folders!"),
            false => {
                println!("Detected {} entries to copy", counter);
                self.compared.maps = entries_to_copy;
                self.compared.generate_metadata(&self.source.metadata.output_folder, &Mode::Cloud);
                Ok(())
            }
        }
    }

    pub fn delete_missing(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Looking for deleted folders and files...");
        let mut deleted: usize = 0;
        if self.backup.maps.is_empty() {
            return Err(Box::try_from("no previous backup found").unwrap())
        }

        for (backup_entries_input, backup_entries) in &self.backup.maps {
            match self.source.maps.get_key_value(backup_entries_input) {
                Some((source_root, source_entries)) => {
                    let root = String::from(source_root);
                    let path = root.as_str();
                    let path_splitted: Vec<&str> = path.split(MAIN_SEPARATOR).collect();
                    let output_to_compare;
                    match path_splitted.last() {
                        Some(val) => {
                            output_to_compare = String::from(&self.backup.metadata.output_folder) + MAIN_SEPARATOR.to_string().as_str() + val;
                        }
                        None => {
                            println!("Error while converting folder paths in {}, skipping...", &root);
                            continue;
                        }
                    }
                    for backup_entry in backup_entries {
                        let mut found = false;
                        let backup_entry_to_compare = backup_entry.path.replacen(&output_to_compare, source_root, 1);
                        for source_entry in source_entries {
                            if backup_entry_to_compare == source_entry.path {
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
                                match self.delete_folder_with_content(&backup_entries, &backup_entry.path) {
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
        if deleted > 0 {
            println!("Deleted {} redundant entries", deleted);
        }
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
                match fs::remove_file(&item.path) {
                    Ok(_) => entries += 1,
                    Err(e) => {
                        println!("Error while removing file {}: {} - skipping...", &item.path, e);
                        continue;
                    }
                }
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

    pub fn copy_compared(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Copying new or modified folders...");

        let mut file_counter: usize = 0;
        let root_destination = self.backup.metadata.output_folder.clone();

        for (compared_root, comparted_entries) in &self.compared.maps {
            if comparted_entries.is_empty() {
                continue;
            }
            println!("Copying new entries from: {}", compared_root);
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

                    // Creating parent folder in case it doesn't exist
                    let current_destination_splitted: Vec<&str> = current_destination.as_str().split(MAIN_SEPARATOR).collect();
                    let current_destination_folder = current_destination.trim_end_matches(current_destination_splitted.last().unwrap());
                    if !Path::new(current_destination_folder).exists() {
                        if let Err(e) = fs::create_dir_all(current_destination_folder) {
                            println!("Couldn't create folder: {}: {}", &current_destination_folder, e);
                        }
                    }

                    match fs::File::create(&current_destination) {
                        Ok(destination_file) => {
                            let mut writer = BufWriter::new(destination_file);
                            if let Err(e) = io::copy(&mut reader, &mut writer) {
                                println!("Couldn't copy file {} to destination {}: {}", &compared_entry.path,  &current_destination, e);
                                self.copying_error_paths.push(current_destination.clone());
                            }
                            file_counter += 1;
                        }
                        Err(e) => {
                            println!("Couldn't create file {}: {}", &compared_entry.path, e);
                            // Deleting file from map
                            self.copying_error_paths.push(compared_entry.path.clone());
                            continue;
                        }
                    }
                }
            }

        }
        if file_counter > 0 {
            println!("Copied {} files", file_counter);
        }
        Ok(())
    }

    pub fn save_json(&mut self) -> Result<(), String> {
        println!("Creating end map of copied files...");
        let mut serialization = Serialization::new();
        let source_maps = self.source.maps.clone();

        for map in source_maps {
            let mut entries_without_errors = map.1.clone();
            for copying_error_path in &self.copying_error_paths {
                entries_without_errors.retain(|x| x.path != copying_error_path.as_str());
            }
            serialization.maps.insert(map.0.clone(), entries_without_errors);
        }

        let output = self.backup.metadata.output_folder.clone();
        serialization.replace_in_paths(&output);

        serialization.generate_metadata(&self.backup.metadata.output_folder, &Mode::Cloud);
        if let Err(e) = serialization.serialize_to_json(&self.backup.metadata.output_folder[..]) {
            let message = String::from("couldn't serialize to JSON: ") + e;
            return Err(message);
        }
        println!("End map created!");
        Ok(())
    }
}