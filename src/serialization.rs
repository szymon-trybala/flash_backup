use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use hex;
use ring::digest::{Context, SHA256};
use digest::Digest;
use meowhash::MeowHasher;
use std::io::{BufReader};
use uuid::Uuid;
use chrono::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: String,
    pub timestamp: i64,
    pub elements: usize,
    pub output_folder: String
}
impl BackupMetadata {
    pub fn new() -> BackupMetadata {
        let backup_metadata = BackupMetadata { id: String::new(), timestamp: 0, elements: 0, output_folder: String::new()};
        backup_metadata
    }
}

#[derive(Serialize, Deserialize)]
pub struct Serialization {
    pub metadata: BackupMetadata,
    pub map: HashMap<String, String>,   // key = path, value = hash
}


impl Serialization {
    pub fn new(paths: Vec<String>) -> Result<Serialization, &'static str> {
        let mut serialization = Serialization { metadata: BackupMetadata::new(), map: HashMap::new()};

        for path in paths {
            // Right now SHA-1 is 1/3 faster than Blake3, but it's bad implementation anyway - 25s for 300 MB file is terrible
            match generate_hash_meow(&path) {
                Ok(hash) => {
                    serialization.map.insert(String::from(path), hash);
                }
                Err(_) => {
                    println!("Couldn't calculate hash of file with path {}, skipping", path);
                    continue;
                }
            }
        }
        Ok(serialization)
    }

    pub fn serialize_to_json(&mut self, save_folder: &str) -> Result<(), &'static str> {
        self.metadata = self.generate_metadata(&save_folder);

        match serde_json::to_string_pretty(&self) {
            Err(_) => Err("Serialization to string failed"),
            Ok(json_string) => {
                let output_path = String::from(save_folder) + "/map.json";
                match File::create(output_path) {
                    Err(_) => Err("Error: couldn't create JSON file with folder map!"),
                    Ok(mut file) => {
                        match file.write_all(json_string.as_ref()) {
                            Err(_) => Err("Error: couldn't write JSON text to file"),
                            Ok(_) => Ok(())
                        }
                    }
                }
            }
        }
    }

    fn generate_metadata(&self, output_folder: &str) -> BackupMetadata {
       BackupMetadata { id: Uuid::new_v4().to_string(), timestamp: Utc::now().timestamp(), elements: self.map.len(), output_folder: output_folder.to_string() }
    }
}

// 8 video files, 2,8 GB combined - 11s of copying + 12s hashing on i5 6200U.
pub fn generate_hash_sha256(path: &String) -> Result<String, &'static str> {
    match File::open(path) {
        Ok(file) => {
           let mut reader = BufReader::new(file);
            let mut context = Context::new(&SHA256);
            let mut buffer = [0; 1024];

            loop {
                let count = reader.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                context.update(&buffer[..count])
            }
            let digest = context.finish();


            Ok(hex::encode(digest.as_ref()))
        }
        Err(_) => Err("Error opening file while generating hash")
    }
}

// ~ 2 GB, 3 files, 2s on Ryzen 3600 <3
pub fn generate_hash_meow(path: &String) -> Result<String, &'static str> {
    match File::open(path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut meow = MeowHasher::new();
            let mut buffer = [0; 1024];

            loop {
                match reader.read(&mut buffer) {
                    Ok(u) => {
                        if u == 0 {
                            break;
                        }
                        meow.input(&buffer[..u]);
                    }
                    Err(_) => {
                        return Err("Error reading chunk of data, skipping file...");
                    }
                }
            }
            let result = meow.result();
            Ok(hex::encode(result.as_ref()))
        }
        Err(_) => Err("Error opening file while generating hash")
    }
}

