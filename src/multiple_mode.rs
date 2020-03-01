use crate::serialization::{BackupMetadata, Serialization};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use walkdir::{WalkDir};
use std::convert::TryFrom;
use chrono::prelude::*;

pub struct Multiple {
    max_backups: usize,
    root_folder: String,
    backups: Vec<BackupMetadata>
}
impl Multiple {
    pub fn new(max_backups: usize, root_folder: String) -> Multiple {
       Multiple { max_backups, root_folder, backups: Vec::new()}
    }

    pub fn create_new_backup_folder(&mut self) -> Result<String, Box<dyn Error>> {
        if let Err(e) = self.find_backups() {
            println!("Error finding previous backups: {}", e);
            return Err(e);
        }

        // Deleting every redundant folder
        loop {
            if self.backups.len() >= self.max_backups {
                if let Err(e) = self.delete_oldest_folder() {
                    println!("Error creating new backup folder: {}", e);
                    return Err(e);
                }
            } else {
                break;
            }
        }

        let now: chrono::DateTime<Utc> = Utc::now();
        let date = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        let new_path = self.root_folder.clone() + "/" + date.as_str();
        Ok(new_path)
    }


    fn find_backups(&mut self) -> Result<(), Box<dyn Error>> {
        let mut vec = Vec::new();
        for entry in WalkDir::new(&self.root_folder).into_iter().filter_map(|e| e.ok()) {
            if entry.path().ends_with("map.json") {
                let file = File::open(entry.path()).unwrap();
                let buf_reader = BufReader::new(file);

                let content: Serialization = serde_json::from_reader(buf_reader)?;
                vec.push(content.metadata);
            }
        }
        self.backups = vec;
        if self.backups.len() == 0 {
            println!("No previous backups found in {}", &self.root_folder);
        }
        Ok(())
    }

    fn delete_oldest_folder(&mut self) -> Result<(), Box<dyn Error>> {
        if self.backups.len() == 0 {
            println!("Error: no backups to delete!");
            return Err(Box::try_from("no backups to delete!").unwrap())
        }

        let mut oldest_timestamp: i64 = i64::max_value();
        let mut oldest_path = String::new();

        for backup in &self.backups {
            if backup.timestamp < oldest_timestamp {
                oldest_timestamp = backup.timestamp;
                oldest_path = backup.output_folder.clone();
            }
        }

        println!("Max amount of backups reached, deleting oldest one: {}", &oldest_path);
        match std::fs::remove_dir_all(&oldest_path) {
            Ok(_) => {
                if let Err(e) = self.find_backups() {
                    println!("Error: deleted folder is still on list, program thinks there are {} backups", self.backups.len());
                    return Err(e);
                }
                Ok(())
            },
            Err(_) => {
                println!("Error removing oldest directory: {}. Program will stop", oldest_path);
                panic!();
            }
        }
    }
}