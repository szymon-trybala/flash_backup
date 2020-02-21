use std::io;
use std::process::exit;

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
    }

    pub fn handle_output_path(&mut self, path_raw: &str) {
        self.output_path = String::from(path_raw.trim());
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
                exit(-1);
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
                exit(-1);
            }
        }
    }
}

