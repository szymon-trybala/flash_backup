use std::path::{Path, MAIN_SEPARATOR};
use std::{fs, io};
use crate::FILE_MAP_NAME;
use std::error::Error;
use std::convert::TryFrom;
use crate::io::serialization::Serialization;
use walkdir::WalkDir;
use crate::modes::Mode;

pub struct Cloud {
    pub existing: Serialization,
    pub new: Serialization,
    pub compared: Vec<String,>
}

impl Cloud {
    pub fn load_existing_serialization(&mut self, folder: &Path) -> Result<(), Box<dyn Error>> {
        if !(folder.exists() && folder.is_dir()) {
            return Err(Box::try_from("invalid selected output path").unwrap());
        }

        // LOADING SERIALIZATION FROM FILE
        let map_path = String::from(folder.to_str().unwrap()) + MAIN_SEPARATOR.to_string().as_ref() + FILE_MAP_NAME;
        let file = fs::File::open(map_path)?;
        let buf_reader = io::BufReader::new(file);
        let map: Serialization = serde_json::from_reader(buf_reader)?;

        self.existing = map;
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
        Ok(())
    }

    pub fn generate_entries_to_copy(&mut self) -> Result<(), &'static str> {
        if self.existing.metadata.mode != Mode::Cloud || self.new.metadata.mode != Mode::Cloud {
            return Err("at least one of maps isn't in cloud mode");
        }
        if self.new.map.is_empty() {
            return Err("no entries to copy");
        }

        let mut entries_to_copy = Vec::new();
        let mut counter: usize = 0;
        for entry in &self.new.map {
            // Only files have hashes
            if entry.0.len() > 0 && !self.existing.map.contains_key(&entry.0[..]) {
                entries_to_copy.push(entry.1.clone());
                counter += 1;
            }
        }

        match entries_to_copy.is_empty() {
            true => Err("both folders are the same"),
            false => {
                println!("Detected {} files to copy", counter);
                self.compared = entries_to_copy;
                Ok(())
            }
        }
    }

    pub fn copy_compared(&self) {
        let from = self.new.metadata.input_folders.clone();
        let to = self.existing.metadata.output_folder.clone();

        for path in from {

        }
    }
}