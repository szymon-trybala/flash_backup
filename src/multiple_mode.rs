use crate::serialization::{BackupMetadata, Serialization};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use walkdir::{WalkDir};
use std::convert::TryFrom;
use chrono::prelude::*;
use std::str::FromStr;

pub struct Multiple {
    root_folder: String,
    backups: Vec<BackupMetadata>
}
impl Multiple {
    pub fn new() -> Multiple {
       Multiple { root_folder: String::new(), backups: Vec::new()}
    }

    pub fn find_backups(&mut self, root_folder: String) -> Result<(), Box<dyn Error>> {
        self.root_folder = root_folder.clone();

        for entry in WalkDir::new(root_folder).into_iter().filter_map(|e| e.ok()) {
            if entry.path().ends_with("map.json") {
                let file = File::open(entry.path()).unwrap();
                let mut buf_reader = BufReader::new(file);

                let content: Serialization = serde_json::from_reader(buf_reader)?;
                self.backups.push(content.metadata);
            }
        }
        Ok(())
    }

    pub fn delete_oldest_folder(&mut self) -> Result<(), Box<dyn Error>> {
        if self.backups.len() == 0 {
            println!("No backups to delete!");
            return Err(Box::try_from("No backups to delete!").unwrap())
        }

        let mut oldest_timestamp: i64 = i64::max_value();
        let mut oldest_path = String::new();

        for backup in &self.backups {
            if backup.timestamp < oldest_timestamp {
                oldest_timestamp = backup.timestamp;
                oldest_path = backup.output_folder.clone();
            }
        }

        match std::fs::remove_dir_all(&oldest_path) {
            Ok(_) => {
                match self.find_backups(self.root_folder.clone()) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("Error: deleted folder is still on list, program thinks there are {} backups", self.backups.len());
                        return Err(e);
                    }
                }
            },
            Err(_) => {
                println!("Error removing oldest directory: {}. Program will stop", oldest_path);
                panic!();
            }
        }
    }
}