mod modes;
mod io;

use crate::modes::multiple_mode::Multiple;
use crate::io::config::Config;
use crate::io::copying::Copying;
use crate::io::serialization::Serialization;
use crate::io::metadata::Metadata;

use std::collections::HashMap;

static FILE_MAP_NAME: &str = ".map.json";
static CONFIG_FILE: &str = ".config.json";

#[cfg(unix)]
pub static FOLDER_SEPARATOR: &str = "/";

#[cfg(windows)]
static FOLDER_SEPARATOR: &str = "\\";

fn main() {
    match Config::new() {
        Err(e) => println!("{}", e),

        Ok(config) => {
            let mut multiple = Multiple::new(config.max_backups, config.output_path.clone());
            match multiple.create_new_backup_folder() {
                Err(_) => panic!("Error while creating new backup folder, program will stop"),

                Ok(new_backup_folder) => {
                    match Copying::new(&config.input_paths) {
                        Err(e) => {
                            let message = String::from("Fatal error while creating file maps: ") + e;
                            panic!(message);
                        }

                        Ok(mut copying) => {
                            match Config::load_ignores() {
                                Err(e) => {
                                    println!("{}", &e);
                                    if let Err(e) = copying.copy(&new_backup_folder) {
                                        let message = "Fatal error while copying files".to_owned() + e;
                                        panic!(message);
                                    }
                                }
                                Ok(ignores) => {
                                    if ignores.0.len() > 0 {
                                        if let Err(e) = copying.exclude_folders(&ignores.0) {
                                            println!("Error while excluding folders, everything will be copied: {}", e);
                                        }
                                    }
                                    if ignores.1.len() > 0 {
                                        if let Err(e) = copying.exclude_files_with_extensions(&ignores.1) {
                                            println!("Error while excluding files with selected extensions, everything will be copied: {}", e);
                                        }
                                    }

                                    if let Err(e) = copying.copy(&new_backup_folder) {
                                        let message = "Fatal error while copying files".to_owned() + e;
                                        panic!(message);
                                    }
                                }
                            }
                            let mut serialization = Serialization::new(copying.output_files_paths).unwrap_or(Serialization { map: HashMap::new(), metadata: Metadata::new() });
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
            }
        }
    }
}