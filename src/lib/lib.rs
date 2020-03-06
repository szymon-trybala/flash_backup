use crate::io::config::Config;
use crate::modes::multiple_mode::Multiple;
use crate::io::copying::Copying;
use crate::io::serialization::Serialization;
use crate::modes::Mode;

pub mod modes;
pub mod io;

static FILE_MAP_NAME: &str = ".map.json";
static CONFIG_FILE: &str = ".config.json";
static IGNORE_FILE: &str = ".ignore";

#[cfg(unix)]
pub static FOLDER_SEPARATOR: &str = "/";

#[cfg(windows)]
static FOLDER_SEPARATOR: &str = "\\";

pub fn create_backup(custom_config: &str, custom_ignore: &str, custom_mode: &str) {
    let config = handle_config(custom_config);
    // TODO - could use Mode trait with backup_folder field
    match custom_mode.trim() {
        "m" | "multiple" => {
            let multiple = handle_multiple_mode(&config);
            let mut copying = handle_maps(&config);
            handle_ignores(custom_ignore, &mut copying);
            handle_copying(&mut copying, &multiple.backup_folder[..]);
            handle_serialization(&config, &copying, &multiple.backup_folder[..]);
        }
        "c" | "cloud" => {
            println!("NOT IMPLEMENTED YET");
        }

        _ => {
            println!("Mode argument not provided, will load it from .config.json");
            match config.mode {
                Mode::Multiple => {
                    println!("Default mode in .config.json is Multiple, executing...");
                    let multiple = handle_multiple_mode(&config);
                    let mut copying = handle_maps(&config);
                    handle_ignores(custom_ignore, &mut copying);
                    handle_copying(&mut copying, &multiple.backup_folder[..]);
                    handle_serialization(&config, &copying, &multiple.backup_folder[..]);
                }
                Mode::Cloud => {
                    println!("Default mode in .config.json is Cloud, executing...");
                    println!("NOT IMPLEMENTED YET");
                }
            }
        }
    }
}

fn handle_config(custom_config: &str) -> Config {
    if let Ok(config) = Config::new(custom_config) {
        config
    } else {
        panic!("Fatal error while loading config");
    }
}

fn handle_multiple_mode(config: &Config) -> Multiple {
    let mut multiple = Multiple::new(config.max_backups, config.output_path.clone());
    if let Ok(_) = multiple.create_new_backup_folder() {
        multiple
    } else {
        panic!("Error while creating new backup folder, program will stop");
    }
}

fn handle_maps(config: &Config) -> Copying {
    match Copying::new(&config.input_paths) {
        Ok(copying) => {
            return copying;
        }
        Err(e) => {
            let message = String::from("Fatal error while creating file maps: ") + e;
            panic!(message);
        }
    }
}

fn handle_ignores(custom_ignore: &str, copying: &mut Copying) {
    match Config::load_ignores(custom_ignore) {
        Err(e) => {
            println!("{}", &e);
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
        }
    }
}

fn handle_copying(copying: &mut Copying, folder: &str) {
    if let Err(e) = copying.copy(&folder) {
        let message = "Fatal error while copying files".to_owned() + e;
        panic!(message);
    }
}

fn handle_serialization(config: &Config, copying: &Copying, folder: &str) {
    if let Ok(mut serialization) = Serialization::new(&copying.output_files_paths) {
        match serialization.serialize_to_json(&config.input_paths, &folder, &config.mode) {
            Ok(_) => {
                println!("JSON file map succesfully saved in root output folder!");
            }
            Err(e) => {
                let message = String::from("Fatal error while trying to serialize maps to JSON: ") + e;
                panic!(message);
            }
        }
    } else {
        panic!("Fatal error while trying to create file with maps");
    }
}