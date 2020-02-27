use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use sha1::{Sha1, Digest};
use hex;
use ring::digest::{Context, Digest as ring_digest, SHA256};

use blake3;
use std::io::BufReader;

const BUFFER_SIZE: usize = 256000;

#[derive(Serialize, Deserialize)]
pub struct Serialization {
    // key = path, value = hash
    pub map: HashMap<String, String>,
}

pub fn generate_hash_sha1(path: &String) -> Result<String, &'static str> {
    let mut hasher = Sha1::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    match File::open(path) {
        Ok(mut file) => {
            loop {
                let n = match file.read(buffer.as_mut()) {
                    Ok(n) => n,
                    Err(_) => return Err("Error reading file while generating hash"),
                };
                hasher.input(&buffer[..n]);
                if n == 0 || n < BUFFER_SIZE {
                    break;
                }
            }
            let hash = hasher.result();
            Ok(hex::encode(hash))
        }
        Err(_) => Err("Error opening file while generating hash")
    }
}

// 8 video files, 2,8 GB combined - 11s of copying + 12s hashing on i5 6200U. Why is this so much faster?
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

pub fn generate_hash_blake3(path: &String) -> Result<String, &'static str> {
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; BUFFER_SIZE];
    match File::open(path) {
        Ok(mut file) => {
            loop {
                let n = match file.read(buffer.as_mut()) {
                    Ok(n) => n,
                    Err(_) => return Err("Error reading file while generating hash"),
                };
                hasher.update(&buffer[..n]);
                if n == 0 || n < BUFFER_SIZE {
                    break;
                }
            }
            let hash = hasher.finalize().to_hex();

            Ok(hash.to_string())
        }
        Err(_) => Err("Error opening file while generating hash")
    }
}

impl Serialization {
    pub fn new(paths: Vec<String>) -> Result<Serialization, &'static str> {
        let mut serialization = Serialization { map: HashMap::new()};

        for path in paths {
            // Right now SHA-1 is 1/3 faster than Blake3, but it's bad implementation anyway - 25s for 300 MB file is terrible
            match generate_hash_sha256(&path) {
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

    pub fn serialize_to_json(&self, save_folder: String) -> Result<(), &'static str> {
        match serde_json::to_string_pretty(&self.map) {
            Err(_) => Err("Serialization to string failed"),
            Ok(json_string) => {
                let output_path = String::from(save_folder + "/map.json");
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
}

