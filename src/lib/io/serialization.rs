use crate::{FOLDER_SEPARATOR, FILE_MAP_NAME};
use crate::io::metadata::Metadata;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use hex;
use ring::digest::{Context, SHA256};
use digest::Digest;
use meowhash::MeowHasher;
use uuid::Uuid;
use chrono::prelude::*;
use crate::modes::Mode;
use walkdir::DirEntry;
use std::path::Path;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub hash: String,
    pub is_file: bool,
}

impl Entry {
    pub fn new(path: &str, hash: &str, is_file: bool) -> Entry {
        Entry { path: path.to_string(), hash: hash.to_string(), is_file }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Serialization {
    pub metadata: Metadata,
    pub map: Vec<Entry>,
}

impl Serialization {
    pub fn new() -> Serialization {
        Serialization { metadata: Metadata::new(), map: Vec::new() }
    }

    pub fn generate_map(&mut self, paths: &Vec<DirEntry>) -> Result<(), &'static str> {
        if paths.is_empty() {
            return Err("no entries provided");
        }

        for path in paths {
            if path.path().is_file() {
                match generate_hash_meow(path.path()) {
                    Ok(hash) => {
                        self.map.push(Entry::new(path.path().to_str().unwrap(), &hash[..], true));
                    }
                    Err(_) => {
                        println!("Couldn't calculate hash of file with path {}, skipping", path.path().to_str().unwrap());
                        continue;
                    }
                }
            } else if path.path().is_dir() {
                self.map.push(Entry::new(path.path().to_str().unwrap(), "", false));
            }
        }
        Ok(())
    }

    pub fn serialize_to_json(&mut self, to: &str) -> Result<(), &'static str> {
        let path = Path::new(to);
        if !(path.exists() && path.is_dir()) {
            return Err("invalid selected path to save map JSON");
        }

        match serde_json::to_string_pretty(&self) {
            Err(_) => Err("Serialization to string failed"),
            Ok(json_string) => {
                let output_path = self.metadata.output_folder.clone() + FOLDER_SEPARATOR + FILE_MAP_NAME;
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

    pub fn generate_metadata(&mut self, input_folders: &Vec<String>, output_folder: &str, mode: &Mode) {
        self.metadata.id = Uuid::new_v4().to_string();
        self.metadata.timestamp = Utc::now().timestamp();
        self.metadata.elements = self.map.len();
        self.metadata.mode = mode.clone();
        self.metadata.output_folder = output_folder.to_string();
        self.metadata.input_folders = input_folders.clone();
    }
}

pub fn generate_hash_sha256(path: &Path) -> Result<String, &'static str> {
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

pub fn generate_hash_meow(path: &Path) -> Result<String, &'static str> {
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

