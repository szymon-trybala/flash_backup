use crate::backups::map::backup_map::BackupMap;
use std::path::Path;
use std::{fs, io};
use crate::backups::map::backup_mode::BackupMode;
use std::io::{BufRead, Write};
use crate::{S_IGNORE, S_CONFIG, S_SEPARATOR};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub input_paths: Vec<String>,
    pub output_path: String,
    pub max_backups: usize,
    pub mode: BackupMode,
}

impl Config {
    pub fn new() -> Config {
        Config { input_paths: vec![], output_path: String::new(), max_backups: 0, mode: BackupMode::Multiple }
    }

    /// Creates BackupMap struct based on initially processed and checked arguments.
    ///
    /// Tries to load existing config file, if no files is found creates new config asking user about data.
    ///
    /// Panics if user's data is wrong.
    pub fn create_backup_map(&mut self, run_new_config: usize, custom_config_path: &str, custom_ignore_path: &str) -> BackupMap {
        let mut config = Config::new();
        println!("RUN NEW CONFIG: {}", run_new_config);
        if run_new_config == 0 {
            match config.load_existing_config(custom_config_path) {
                Ok(custom_config) => config = custom_config,
                Err(_) => {
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
        match self.load_ignores(ignore_path) {
            Err(e) => println!("{}", e),
            Ok((folders, extensions)) => {
                map.ignore_folders = folders;
                map.ignore_extensions = extensions;
            }
        }
        map
    }

    /// Loads existing config file in the same folder as program executable, converts it to Config struct, then returns it.
    ///
    /// May return error if file can't be opened or file can't be converted to struct.
    pub fn load_existing_config(&mut self, custom_config_path: &str) -> Result<Config, String> {
        let config_path;
        if custom_config_path.len() > 0 {
            config_path = custom_config_path;
        } else {
            config_path = S_CONFIG;
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

    /// Creates new config struct from user input, then saves it to .JSON file.
    ///
    /// Panics if data provided by user isn't valid.
    pub fn create_and_save_config(&mut self) -> Config {
        println!("Couldn't find config file, create one:");
        let mut config = Config {output_path: String::new(), input_paths: vec![], max_backups: 0, mode: BackupMode::Multiple };
        config.input_paths = self.get_input_paths_from_user();
        config.output_path = self.get_output_path_from_user();
        config.mode = self.get_mode_from_user();
        match config.mode {
            BackupMode::Cloud => {
                config.max_backups = 1;
            }
            BackupMode::Multiple => {
                config.max_backups = self.get_max_backups_amount_from_user();
            }
        }
        if let Err(_) = self.save_config_to_json(&config) {
            println!("Couldn't write config to file, config won't be saved");
        }
        config
    }

    /// Checks if path exists, then returns trimmed string.
    ///
    /// Should be used only from ```get_input_paths_from_user``` function.
    ///
    /// May return error if path doesn't exist.
    pub fn check_input_path_from_user(&mut self, path_raw: &str) -> Result<String, String> {
        let trimmed = path_raw.trim();
        match Path::new(&trimmed).exists() {
            true => Ok(String::from(trimmed)),
            false => Err(String::from("This input path doesn't exist"))
        }
    }

    /// Saves provided ```Config``` struct to JSON file with name the same as ```S_CONFIG``` const.
    ///
    /// May return error if serialization fails, file can't be opened or text can't be writed.
    pub fn save_config_to_json(&self, config: &Config) -> Result<(), &'static str> {
        match serde_json::to_string_pretty(config) {
            Err(_) => Err("Serialization to string failed"),
            Ok(json_string) => {
                match fs::File::create(S_CONFIG) {
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

    /// Asks user about input folders, checks if it's valid, then asks user if he wants to add another. If user agrees, process repeats.
    ///
    /// If user points to non-valid path or error occurs, user is asked to input path again.
    pub fn get_input_paths_from_user(&mut self) -> Vec<String> {
        let mut input_paths = vec![];

        loop {
            let mut path = String::new();
            println!("Write path to source folder:");
            match io::stdin().read_line(&mut path) {
                Ok(_) => {
                    if path.trim().len() == 0 {
                        println!("Input path is empty!");
                        self.get_input_paths_from_user();
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


    /// Asks user about output folder, checks if it's valid, if yes folder's path is returned.
    ///
    /// If user points to non-valid path or error occurs, user is asked to input path again.
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
            Err(_) => {
                println!("Can't read your output ");
                return self.get_output_path_from_user();
            }
        }
    }

    /// Asks user about maximum amount of backups (only for Multiple mode), checks if it's valid, if yes, amount is returned.
    ///
    /// If user writes invalid string, he's asked to do it again.
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

    /// Asks user about mode of backup (m/multiple or c/cloud), checks if it's valid, if yes, amount is returned.
    ///
    /// If user writes invalid string, he's asked to do it again.
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

    /// Loads ignores from default path (the same as program's executable) or provided path if it's non-empty, then returns it as tuple of two vectors.
    ///
    /// Ignore syntax is ".extension" for extensions (for example ".exe") and "/folder" for folders, with slash on all operating systems. Example: "/node_modules".
    ///
    /// May return error if file can't be found, other errors are printed to user
    pub fn load_ignores(&self, custom_ignore: &str) -> Result<(Vec<String>, Vec<String>), &'static str>  {
        let ignore_path;
        if custom_ignore.len() > 0 {
            ignore_path = custom_ignore;
        } else {
            ignore_path = S_IGNORE;
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
                            } else if line.starts_with(S_SEPARATOR) {
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