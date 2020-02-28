use std::io;
use std::fs;
use std::path::Path;
use std::process;
use std::fs::File;
use std::io::{BufReader, BufRead};

pub struct Paths {
    pub input_path: String,
    pub output_path: String,
}

impl Paths {
    pub fn new() -> Paths {
        let mut paths = Paths { input_path: String::new(), output_path: String::new() };
        paths.ask_for_input();
        paths.ask_for_output();
        paths
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
                if path.len() == 0 {
                    println!("Input path is empty!");
                    process::exit(-1);
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
                if path.len() == 0 {
                    println!("Input path is empty!");
                    process::exit(-1);
                }
            }
            Err(_error) => {
                println!("Error reading path to destination folder");
                process::exit(-1);
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
