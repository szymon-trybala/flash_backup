use crate::{FOLDER_SEPARATOR, FILE_MAP_NAME};
use crate::io::metadata::Metadata;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File};
use std::io::prelude::*;
use std::io::{BufReader};
use hex;
use ring::digest::{Context, SHA256};
use digest::Digest;
use meowhash::MeowHasher;
use uuid::Uuid;
use chrono::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Serialization {
    pub metadata: Metadata,
    pub map: HashMap<String, String>,   // key = path, value = hash
}

impl Serialization {
    pub fn new(paths: Vec<String>) -> Result<Serialization, &'static str> {
        let mut serialization = Serialization { metadata: Metadata::new(), map: HashMap::new()};

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

    pub fn serialize_to_json(&mut self, input_folders: &Vec<String>, save_folder: &str) -> Result<(), &'static str> {
        self.metadata = self.generate_metadata(input_folders, &save_folder);

        match serde_json::to_string_pretty(&self) {
            Err(_) => Err("Serialization to string failed"),
            Ok(json_string) => {
                let output_path = String::from(save_folder) + FOLDER_SEPARATOR + FILE_MAP_NAME;
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

    fn generate_metadata(&self, input_folders: &Vec<String> ,output_folder: &str) -> Metadata {
       Metadata { id: Uuid::new_v4().to_string(), timestamp: Utc::now().timestamp(), elements: self.map.len(), output_folder: output_folder.to_string(), input_folders: input_folders.clone() }
    }
}

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

