use std::path::{Path, MAIN_SEPARATOR};
use std::{fs, io};
use crate::FILE_MAP_NAME;
use std::error::Error;
use crate::io::serialization::{Serialization};
use walkdir::WalkDir;
use crate::modes::Mode;
use std::io::{BufReader, BufWriter};

pub struct Cloud {
    pub existing: Serialization,
    pub new: Serialization,
    pub compared: Serialization,
}

impl Cloud {
    pub fn new() -> Cloud {
        Cloud {existing: Serialization::new(), new: Serialization::new(), compared: Serialization::new()}
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

        self.existing = map;
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

        self.new = serialization;
        self.compared.metadata.input_folders.push(input_path.to_str().unwrap().to_string());
        Ok(())
    }

    pub fn generate_entries_to_copy(&mut self) -> Result<(), &'static str> {
        // TODO - TERRIBLE PERFORMANCE, FIX IT
        if self.new.metadata.mode != Mode::Cloud {
            return Err("at least one of maps isn't in cloud mode");
        }
        if self.new.map.is_empty() {
            return Err("no entries to copy");
        }

        let mut entries_to_copy = Vec::new();
        let mut counter: usize = 0;
        for entry in &self.new.map {
            if entry.is_file {
                let mut found_matching_hash = false;
                for old_entry in &self.existing.map {
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
                for old_entry in &self.existing.map {
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
                println!("Detected {} entries to copy", counter - 1);
                self.compared.map = entries_to_copy;
                Ok(())
            }
        }
    }

    pub fn copy_compared(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO - SKIPPING ROOT OUTPUT FOLDER SHOULD BE DONE BEFORE, FIX IT
        let mut file_counter: usize = 0;
        let mut folder_counter: usize = 0;

        let root_destination = self.existing.metadata.output_folder.clone();
        let root_source = self.new.metadata.output_folder.clone();
        if self.compared.map[0].path == root_destination {
            self.compared.map.remove(0);
        }

        for current_source in &self.compared.map {
            let current_destination = current_source.path.replacen(root_source.as_str(), &root_destination[..], 1);

            // CREATE FOLDER
            if !current_source.is_file {
                fs::create_dir_all(&current_destination)?;
                folder_counter += 1;
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
        println!("Created {} folders and copied {} files", folder_counter, file_counter);
        Ok(())
    }

    pub fn save_json(&mut self) {
        // TODO - TERRIBLE PERFORMANCE, UNNECESSERY COPYING, FIX IT
        // Creating new map of folder after all changes
        let mut entries = Vec::new();
        for copied_entry in WalkDir::new(&self.existing.metadata.output_folder).into_iter().filter_map(|e| e.ok()) {
            entries.push(copied_entry);
        }
        let mut serialization = Serialization::new();
        serialization.generate_map(&entries);
        serialization.generate_metadata(&self.existing.metadata.input_folders, &self.existing.metadata.output_folder, &Mode::Cloud);
        serialization.serialize_to_json(&self.existing.metadata.output_folder[..]);
    }
}