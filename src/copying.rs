use walkdir::{WalkDir, DirEntry};
use std::path::Path;
use std::fs;
use std::process::exit;
use std::io;

pub struct Copying {
    source_file_tree: Vec<DirEntry>,
}

impl Copying {
    pub fn new() -> Copying {
        Copying { source_file_tree: Vec::new() }
    }

    pub fn copy(&mut self, from: &str, to: &str) {
        // Creating recursive tree of files in source folder using walkdir library
        // TODO: improve filter_map, add error handling
        for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
            self.source_file_tree.push(entry);
        }

        // Creating destination folder
        if !Path::new(to).exists() {
            match fs::create_dir_all(to) {
                Err(_) => {
                    println!("Error creating destination folder");
                    exit(-1);
                }
                _ => {}
            }
        }

        // First element of vector is always base folder path
        let base_input_path = self.source_file_tree[0].path().to_str().expect("Error converting source base path (self.source_file_tree[0].path()) to &str");
        let base_output_path = to;

        // Copying
        for current_source in self.source_file_tree[1..].iter() {
            let current_source_path = current_source.path().to_str().expect("Error converting source path (&DirEntry.path()) to &str");
            let current_destination_path = current_source_path.replacen(base_input_path, base_output_path, 1);

            if current_source.path().is_dir() {
                match fs::create_dir_all(&current_destination_path) {
                    Err(_) => {
                        println!("Error creating new directory: {}", &current_destination_path);
                    }
                    _ => {}
                }
            }
            else if current_source.path().is_file() {
                //TODO: find a way to get rid of all this Err(_)
                match fs::File::open(&current_source_path) {
                    Ok(mut source_file) => {
                        match fs::File::create(&current_destination_path) {
                            Ok(mut destination_file) => {
                                // io::copy was faster then fs::copy, because it's writing data "in streaming fashion"
                                match io::copy(&mut source_file, &mut destination_file) {
                                    Err(_) => {
                                        println!("Error copying to destination: {}", &current_destination_path);
                                        match fs::remove_file(&current_destination_path) {
                                            Err(_) => {
                                                println!("Error deleting empty file: {}", &current_destination_path);
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Err(_) => {
                                println!("Error creating file: {}", &current_destination_path);
                            }
                        }
                    }
                    Err(_) => {
                        println!("Error reading source file: {}", &current_source_path);
                    }
                }
            }
        }
        println!("Copying finished succesfully!");
    }
}