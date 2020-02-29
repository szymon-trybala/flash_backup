use std::io;
use std::fs;
use std::path::Path;
use std::process;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::error::Error;
use serde_json;
use serde::{Deserialize, Serialize};



static CONFIG_FILE: &str = "config.json";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub input_path: String,
    pub output_path: String,
    pub max_backups: usize,
}

impl Config {
    pub fn new() -> Config {
        let mut paths = Config { input_path: String::new(), output_path: String::new(), max_backups: 3 };
        match paths.load_config() {
            Ok(_) => paths,
            Err(e) => panic!("Error loading config: {}", e.to_string())
        }
    }

    pub fn load_config(&mut self) -> Result<(), Box<dyn Error>> {
        if Path::new(CONFIG_FILE).exists() {
            let mut file = File::open(CONFIG_FILE)?;
            let buf_reader = BufReader::new(file);

            let content: Config = serde_json::from_reader(buf_reader)?;
            self.input_path = content.input_path;
            self.output_path = content.output_path;
            self.max_backups = content.max_backups;
            Ok(())
        } else {
            self.ask_for_config();
            Ok(())
        }
    }

    // TODO - duplicates with saving map, can be reafactored
    pub fn save_config_to_json(&mut self) -> Result<(), &'static str> {
        match serde_json::to_string_pretty(&self) {
            Err(_) => Err("Serialization to string failed"),
            Ok(json_string) => {
                match File::create(CONFIG_FILE) {
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

    pub fn ask_for_config(&mut self) {
        self.ask_for_input();
        self.ask_for_output();
        self.ask_for_max_backups_amount();

        if let Err(_) = self.save_config_to_json() {
            println!("Error writing config to file, config won't be saved!");
        }
    }

    pub fn handle_input_path(&mut self, path_raw: &str) {
        self.input_path = String::from(path_raw.trim());
        if !Path::new(&self.input_path).exists() {
            println!("This input path doesn't exist, program will shut down");
            process::exit(-1);
        }
    }

    pub fn handle_output_path(&mut self, path_raw: &str) {
        self.output_path = String::from(path_raw.trim());
        if !Path::new(&self.output_path).exists() {
            println!("Output folder doesn't exist, creating...");
            if let Err(_) = fs::create_dir_all(&self.output_path) {
                println!("Error creating destination folder");
                process::exit(-1);
            }
            println!("Output folder created!")
        }
    }

    pub fn ask_for_input(&mut self) {
        let mut path = String::new();

        println!("Write path to source folder:");

        match io::stdin().read_line(&mut path) {
            Ok(_n) => {
                self.handle_input_path(&path[..]);
                if path.trim().len() == 0 {
                    println!("Input path is empty!");
                    self.ask_for_input();
                }
            }
            Err(_error) => {
                println!("Error reading path to source folder");
                process::exit(-1);
            }
        }
    }

    pub fn ask_for_output(&mut self) {
        let mut path = String::new();

        println!("Write path to destination folder:");

        match io::stdin().read_line(&mut path) {
            Ok(_n) => {
                self.handle_output_path(&path[..]);
                if path.trim().len() == 0 {
                    println!("Input path is empty!");
                    self.ask_for_output();
                }
            }
            Err(_) => {
                panic!("Error reading path to destination folder");
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
                panic!("Error reading input for max backups amount")
            }
        }

    }

    pub fn load_ignores() -> Result<(Vec<String>, Vec<String>), &'static str>  {
        return match File::open("ignore") {
            Err(_) => {
                Err("Couldn't find ignore file, please make sure it's in Flash Backup root folder")
            }
            Ok(file) => {
                let mut ignores_folders = Vec::new();
                let mut ignores_extensions = Vec::new();
                let reader = BufReader::new(file);

                for (index, line) in reader.lines().enumerate() {
                    match line {
                        Err(_) => {
                            println!("Couldn't read line {}, it will be skipped", &index);
                            continue;
                        }
                        Ok(line) => {
                            // Lines for file extensions has to start with dot and not end with / or \
                            if line.starts_with(".") {
                                ignores_extensions.push(line);
                            } else if line.starts_with("/") || line.starts_with("\\") {
                                ignores_folders.push(line);
                            }
                        }
                    }
                }
                println!("Ignoring {} folders and {} file extensions", ignores_folders.len(), ignores_extensions.len());
                return Ok((ignores_folders, ignores_extensions));
            }
        }
    }
}
