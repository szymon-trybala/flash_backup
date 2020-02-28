use crate::serialization::{BackupMetadata, Serialization};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use walkdir::{WalkDir, DirEntry};
use std::fs;
use std::io;
use std::path::Path;

pub struct Multiple {
    backups: Vec<BackupMetadata>
}
impl Multiple {
    pub fn new() -> Multiple {
       Multiple { backups: Vec::new()}
    }

    pub fn find_backups(&mut self, root_folder: String) -> Result<(), Box<dyn Error>> {
        for entry in WalkDir::new(root_folder).into_iter().filter_map(|e| e.ok()) {
            if entry.path().ends_with("map.json") {
                let file = File::open(entry.path()).unwrap();
                let mut buf_reader = BufReader::new(file);

                let content: Serialization = serde_json::from_reader(buf_reader).unwrap();
                self.backups.push(content.metadata);
            }
        }
        Ok(())
    }
}