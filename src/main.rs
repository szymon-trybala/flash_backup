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
    // TODO AT FINISH CHECK ALL unwrap() and expect()
    // TODO, MOVE EVERYTHING DO LOGICAL MODULES, NOW CODE IS ALL OVER THE PLACE
    println!("Hello, to ignore desired folders or file extensions fill up 'ignore' file in program directory");
    let config = Config::new();
    let mut multiple = Multiple::new(config.max_backups, config.output_path.clone());
    match multiple.create_new_backup_folder() {
        Err(_) => panic!("Error while creating new backup folder, program will stop"),

        Ok(new_backup_folder) => {
            let mut copying = Copying::new(config.input_path.as_str());

            match Config::load_ignores() {
                Err(e) => println!("{}", &e),

                Ok(ignores) => {
                    if ignores.0.len() > 0 {
                        if let Err(_) = copying.exclude_folder(&ignores.0) {
                            println!("Error loading folders to ignore, program will copy every folder!");
                        }
                    }
                    if ignores.1.len() > 0 {
                        if let Err(_) = copying.exclude_files_with_extension(&ignores.1) {
                            println!("Error loading extensions to ignore, program will copy files with every extensions!");
                        }
                    }
                }
            }
            copying.copy(&new_backup_folder);
            let mut serialization = Serialization::new(copying.output_files_paths).unwrap_or(Serialization { map: HashMap::new(), metadata: BackupMetadata::new()});
            match serialization.serialize_to_json(&new_backup_folder) {
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