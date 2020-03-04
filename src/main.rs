mod config;
mod copying;
mod serialization;
mod multiple_mode;

use config::Config;
use crate::copying::Copying;
use crate::serialization::{Serialization, BackupMetadata};
use std::collections::HashMap;
use crate::multiple_mode::Multiple;

fn main() {
    let config = Config::new();
    let mut multiple = Multiple::new(config.max_backups, config.output_path.clone());
    match multiple.create_new_backup_folder() {
        Err(_) => panic!("Error while creating new backup folder, program will stop"),

        Ok(new_backup_folder) => {
            let mut copying = Copying::new(&config.input_paths);

            match Config::load_ignores() {
                Err(e) => {
                    println!("{}", &e);
                    if let Err(e) = copying.copy(&new_backup_folder) {
                        let message = "Fatal error while copying files".to_owned() + e;
                        panic!(message);
                    }
                }
                Ok(ignores) => {
                    if let Err(e) = copying.exclude_folders(&ignores.0) {
                        println!("Error while excluding folders, everything will be copied: {}", e);
                    }
                    if let Err(e) = copying.exclude_files_with_extensions(&ignores.1) {
                        println!("Error while excluding files with selected extensions, everything will be copied: {}", e);
                    }

                    if let Err(e) = copying.copy(&new_backup_folder) {
                        let message = "Fatal error while copying files".to_owned() + e;
                        panic!(message);
                    }
                }
            }
            let mut serialization = Serialization::new(copying.output_files_paths).unwrap_or(Serialization { map: HashMap::new(), metadata: BackupMetadata::new() });
            match serialization.serialize_to_json(&config.input_paths, &new_backup_folder) {
                Ok(_) => {
                    println!("JSON file map succesfully saved in root output folder!");
                }
                Err(e) => {
                    println!("Serialization to JSON: {}", e);
                }
            }
        }
    }
}