use std::io;
use std::fs;
use std::path::Path;
use std::process;

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
            }
            Err(_error) => {
                println!("Error reading path to destination folder");
                process::exit(-1);
            }
        }
    }
}
