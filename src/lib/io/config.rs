use crate::{FOLDER_SEPARATOR, CONFIG_FILE, IGNORE_FILE};

use std::io;
use std::fs;
use std::path::Path;
use std::error::Error;
use serde_json;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::{Write, BufRead};
use crate::modes::{Mode};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub input_paths: Vec<String>,
    pub output_path: String,
    pub max_backups: usize,
    pub mode: Mode,
}

impl Config {
    pub fn new(custom_config: &str) -> Result<Config, Box<dyn Error>> {
        let mut config = Config { input_paths: Vec::new(), output_path: String::new(), max_backups: 3, mode: Mode::Multiple };
        match config.load_config(custom_config) {
            Ok(_) => Ok(config),
            Err(e) => Err(e)
        }
    }

    pub fn load_config(&mut self, custom_config: &str) -> Result<(), Box<dyn Error>> {
        let config_path;
        if custom_config.len() > 0 {
            config_path = custom_config;
        } else {
            config_path = CONFIG_FILE;
        }
        if Path::new(config_path).exists() {
            let file = fs::File::open(config_path)?;
            let buf_reader = io::BufReader::new(file);

            let content: Config = serde_json::from_reader(buf_reader)?;
            self.input_paths = content.input_paths;
            self.output_path = content.output_path;
            self.max_backups = content.max_backups;
            self.mode = content.mode;
            Ok(())
        } else {
            match self.ask_for_config() {
                Err(e) => {
                    let message = String::from("Error: ") + e;
                    return Err(Box::try_from(message).unwrap())
                }
                Ok(_) => Ok(())
            }
        }
    }

    // TODO - duplicates with saving map, can be reafactored
    pub fn save_config_to_json(&mut self) -> Result<(), &'static str> {
        match serde_json::to_string_pretty(&self) {
            Err(_) => Err("Serialization to string failed"),
            Ok(json_string) => {
                match fs::File::create(CONFIG_FILE) {
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

    pub fn ask_for_config(&mut self) -> Result<(), &'static str> {
        println!("Couldn't find config file, create one:");
        self.ask_for_input();
        self.ask_for_output();
        self.ask_for_max_backups_amount();
        self.ask_for_mode();

        if let Err(_) = self.save_config_to_json() {
            return Err("couldn't write config to file, config won't be saved!");
        }
        Ok(())
    }

    pub fn handle_input_path(&mut self, path_raw: &str) -> Result<(), &'static str> {
        let trimmed = path_raw.trim();
        if !Path::new(&trimmed).exists() {
            return Err("this input path doesn't exist");
        } else {
            self.input_paths.push(String::from(trimmed));
        }
        Ok(())
    }

    pub fn handle_output_path(&mut self, path_raw: &str) -> Result<(), &'static str> {
        self.output_path = String::from(path_raw.trim());
        if !Path::new(&self.output_path).exists() {
            println!("Output folder doesn't exist, creating...");
            if let Err(_) = fs::create_dir_all(&self.output_path) {
                return Err("couldn't create destination folder");
            }
            println!("Output folder created!");
        }

        Ok(())
    }

    pub fn ask_for_input(&mut self) {
        loop {
            let mut path = String::new();
            println!("Write path to source folder:");
            match io::stdin().read_line(&mut path) {
                Ok(_) => {
                    if let Err(e) = self.handle_input_path(&path[..]) {
                        println!("Error: {}. Asking again...", e);
                        continue;
                    }
                    if path.trim().len() == 0 {
                        println!("Input path is empty!");
                        self.ask_for_input();
                    }
                }
                Err(_) => {
                    println!("Error reading path to source folder");
                    self.ask_for_input();
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
    }

    pub fn ask_for_output(&mut self) {
        let mut path = String::new();
        println!("Write path to destination folder:");

        match io::stdin().read_line(&mut path) {
            Ok(_n) => {
                if let Err(e) = self.handle_output_path(&path[..]) {
                    let message = String::from("Couldn't handle your output folder: ") + e + ". Program will exit";
                    panic!(message);
                }
                if path.trim().len() == 0 {
                    println!("Input path is empty!");
                    self.ask_for_output();
                }
            }
            Err(_) => {
                println!("Error reading path to destination folder, asking again...");
                self.ask_for_output();
            }
        }
    }

    pub fn ask_for_max_backups_amount(&mut self) {
        let mut amount = String::new();
        println!("How many backups do you want to keep?:");

        match io::stdin().read_line(&mut amount) {
            Ok(_) => {
                if amount.trim().len() == 0 {
                    println!("Input string is empty!");
                    self.ask_for_max_backups_amount();
                }

                let amount: usize = amount.parse().unwrap_or(3);
                self.max_backups = amount;
            }
            Err(_) => {
                println!("Error reading input for max backups amount, asking again...");
                self.ask_for_max_backups_amount();
            }
        }

    }

    pub fn ask_for_mode(&mut self) {
        let mut mode = String::new();
        println!("Do you want to use multiple or cloud mode (m/c)?:");
        if let Err(_) = io::stdin().read_line(&mut mode) {
            println!("Error reading input for mode, asking again...");
            self.ask_for_mode();
        }

        match mode.trim() {
            "m" | "multiple" => {
                self.mode = Mode::Multiple;
            },
            "c" | "cloud" => {
                self.mode = Mode::Cloud;
            },

            _ => {
                println!("Wrong input provided, please write 'm' for multiple mode or 'c' for cloud mode");
                self.ask_for_mode();
            }
        }
    }

    pub fn load_ignores(custom_ignore: &str) -> Result<(Vec<String>, Vec<String>), &'static str>  {
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
