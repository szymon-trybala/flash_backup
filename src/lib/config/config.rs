use crate::backups::map::backup_map::BackupMap;
use std::path::Path;
use std::{fs, io};
use crate::backups::map::backup_mode::BackupMode;
use std::io::BufRead;
use crate::S_IGNORE;

#[derive(Clone)]
pub struct Config {
    pub input_paths: Vec<String>,
    pub output_path: String,
    pub max_backups: usize,
    pub mode: Mode,
}

impl Config {
    pub fn new() -> Config {
        Config { input_paths: vec![], output_path: String::new(), max_backups: 0, mode: BackupMode::Multiple }
    }

    pub fn create_backup_map(&mut self, run_new_config: usize, custom_config_path: &str, custom_ignore_path: &str, mode: &str) -> BackupMap {
        let mut config = Config::new();

        if run_new_config == 0 {
            match config.load_existing_config(custom_config_path) {
                Ok(custom_config) => config = custom_config,
                Err(e) => {
                    println!("Error loading config, asking again...");
                    config = self.create_and_save_config();
                }
            }
        } else {
            config = self.create_and_save_config();
        }
        let mut map = BackupMap { max_backups: config.max_backups, output_folder: config.output_path, input_folders: config.input_paths, backup_mode: config.mode, backup_dirs: vec![], files: 0, folders: 0, timestamp: 0, id: String::new(), ignore_folders: vec![], ignore_extensions: vec![] };

        let ignore_path;
        if custom_ignore_path.is_empty() {
            ignore_path = S_IGNORE;
        } else {
            ignore_path = custom_ignore_path;
        }
        let ignores;
        match self.load_ignores(ignore_path) {
            Err(e) => println!("{}", e),
            Ok((folders, extensions)) => {
                map.ignore_folders = folders;
                map.ignore_extensions = extensions;
            }
        }
        BackupMap
    }

    pub fn load_existing_config(&mut self, custom_config_path: &str) -> Result<Config, String> {
        let config_path;
        if custom_config_path.len() > 0 {
            config_path = custom_config_path;
        } else {
            config_path = CONFIG_FILE;
        }

        match Path::new(config_path).exists() {
            true => {
                match fs::File::open(config_path) {
                    Err(e) => {
                        let message = format!("Can't open file with config path {}: {}", config_path, e);
                        return Err(message);
                    }
                    Ok(file) => {
                        let buf_reader = io::BufReader::new(file);
                        match serde_json::from_reader(buf_reader) {
                            Err(e) => {
                                let message = format!("Can't read config from file file with path {}: {}", config_path, e);
                                return Err(message);
                            }
                            Ok(config) => Ok(config)
                        }
                    }
                }
            }
            false => {
                let message = format!("Config path {} doesn't exist", config_path);
                return Err(message);
            }
        }
    }

    pub fn create_and_save_config(&mut self) -> Config {
        println!("Couldn't find config file, create one:");
        let mut config = Config {output_path: String::new(), input_paths: vec![], max_backups: 0, mode: BackupMode::Multiple };
        config.input_paths = self.get_input_paths_from_user();
        config.output_path = self.get_output_path_from_user();
        config.mode = self.get_mode_from_user();
        match self.mode {
            BackupMode::Cloud => {
                config.max_backups = 1;
            }
            BackupMode::Multiple => {
                config.max_backups = self.get_max_backups_amount_from_user();
            }
        }

        if let Err(_) = self.save_config_to_json() {
            panic!("Couldn't write config to file, config won't be saved, program will stop");
        }
        config
    }

    pub fn check_input_path_from_user(&mut self, path_raw: &str) -> Result<String, String> {
        let trimmed = path_raw.trim();
        match Path::new(&trimmed).exists() {
            true => Ok(String::from(trimmed)),
            false => Err(String::from("This input path doesn't exist"))
        }
    }

    pub fn get_input_paths_from_user(&mut self) -> Vec<String> {
        let mut input_paths = vec![];

        loop {
            let mut path = String::new();
            println!("Write path to source folder:");
            match io::stdin().read_line(&mut path) {
                Ok(_) => {
                    if path.trim().len() == 0 {
                        println!("Input path is empty!");
                        self.ask_for_input();
                    }

                    match self.check_input_path_from_user(&path[..]) {
                        Err(e) => {
                            println!("Error: {}. Asking again...", e);
                            continue;
                        },
                        Ok(path) => input_paths.push(path),
                    }

                }
                Err(_) => {
                    println!("Error reading path to source folder");
                    self.get_input_paths_from_user();
                }
            }

            let mut answer = String::new();
            println!("Would you like to add another folder? (y/n):");
            match io::stdin().read_line(&mut answer) {
                Ok(_) => {
                    let trimmed = answer.trim();
                    if trimmed == "y" || trimmed == "yes" {
                        continue;
                    } else {
                        break;
                    }
                }
                Err(_) => {
                    println!("Error processing your answer, asking again...");
                    continue;
                }
            }
        }
        input_paths
    }

    pub fn get_output_path_from_user(&mut self) -> String {
        let mut path = String::new();
        println!("Write path to destination folder:");

        match io::stdin().read_line(&mut path) {
            Ok(_) => {
                if path.trim().len() == 0 {
                    println!("Input path is empty!");
                    return self.get_output_path_from_user();
                } else {
                    return String::from(path.trim());
                }
            },
            Err(e) => {
                println!("Can't read your output ");
                return self.get_output_path_from_user();
            }
        }
        String::new()
    }

    pub fn get_max_backups_amount_from_user(&mut self) -> usize {
        let mut amount = String::new();
        println!("How many backups do you want to keep?:");

        match io::stdin().read_line(&mut amount) {
            Ok(_) => {
                if amount.trim().len() == 0 {
                    println!("Input string is empty!");
                    return self.get_max_backups_amount_from_user();
                }

                match amount.trim().parse::<usize>() {
                    Ok(amount) => {
                        return amount;
                    }
                    Err(_) => {
                        println!("Error reading input for max backups amount, asking again...");
                        return self.get_max_backups_amount_from_user();
                    }
                }
            }
            Err(_) => {
                println!("Error reading input for max backups amount, asking again...");
                self.get_max_backups_amount_from_user();
            }
        }
        1
    }

    pub fn get_mode_from_user(&mut self) -> BackupMode {
        let mut mode = String::new();
        println!("Do you want to use multiple or cloud mode (m/c)?:");
        if let Err(_) = io::stdin().read_line(&mut mode) {
            println!("Error reading input for mode, asking again...");
            self.get_mode_from_user();
        }

        match mode.trim() {
            "m" | "multiple" => BackupMode::Multiple,
            "c" | "cloud" => BackupMode::Cloud,
            _ => {
                println!("Wrong input provided, please write 'm' for multiple mode or 'c' for cloud mode");
                return self.get_mode_from_user();
            }
        }
    }

    pub fn load_ignores(&self, custom_ignore: &str) -> Result<(Vec<String>, Vec<String>), &'static str>  {
        let ignore_path;
        if custom_ignore.len() > 0 {
            ignore_path = custom_ignore;
        } else {
            ignore_path = IGNORE_FILE;
        }

        return match fs::File::open(ignore_path) {
            Err(_) => {
                Err("No .ignore file found")
            }
            Ok(file) => {
                let mut ignores_folders = Vec::new();
                let mut ignores_extensions = Vec::new();
                let reader = io::BufReader::new(file);

                for (index, line) in reader.lines().enumerate() {
                    match line {
                        Err(_) => {
                            println!("Couldn't read line {} of .ignore file, it will be skipped", &index);
                            continue;
                        }
                        Ok(line) => {
                            // Lines for file extensions has to start with dot and not end with / or \
                            if line.starts_with(".") {
                                ignores_extensions.push(line);
                            } else if line.starts_with(FOLDER_SEPARATOR) {
                                ignores_folders.push(line);
                            }
                        }
                    }
                }
                println!("Found {} folders and {} file extensions in .ignore file", ignores_folders.len(), ignores_extensions.len());
                return Ok((ignores_folders, ignores_extensions));
            }
        }
    }
}