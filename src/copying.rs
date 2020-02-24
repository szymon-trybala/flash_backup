use walkdir::{WalkDir, DirEntry};
use std::fs;
use std::io;
use std::path::Path;

pub struct Copying {
    source_file_tree: Vec<DirEntry>,
}

impl Copying {
    pub fn new(source_folder: &str) -> Copying {
        let mut copying = Copying { source_file_tree: Vec::new() };
        // Creating recursive tree of files in source folder using walkdir library
        // TODO: improve filter_map, add error handling
        for entry in WalkDir::new(source_folder).into_iter().filter_map(|e| e.ok()) {
            copying.source_file_tree.push(entry);
        }
        copying
    }

    pub fn exclude_folder(&mut self, folders: &Vec<String>) -> Result<(), &'static str> {
        if self.source_file_tree.is_empty() {
            return Err("Error: folder tree is empty");
        }

        for entry in folders {
            // Checking if path exists
            let mut full_entry_path =  String::from(self.source_file_tree[0].path().to_str().unwrap());
            full_entry_path.push_str(&entry);

            let path = Path::new(&full_entry_path);
            if !path.exists() {
                println!("{} doesn't exist, skipping", path.to_str().unwrap());
            }

            // Checking if path to exclude is folder
            if !path.is_dir() {
               println!("{} isn't directory, skipping", path.to_str().unwrap());
                continue;
            }
            self.source_file_tree.retain(|x|
                !x.path().to_str().expect("Error while converting path {} to string").contains(&full_entry_path))
        }
        Ok(())
    }

    pub fn exclude_files_with_extension(&mut self, extensions: &Vec<String>) -> Result<(), &'static str> {
        if self.source_file_tree.is_empty() {
            return Err("File tree is empty!");
        }

        let start_len = self.source_file_tree.len();
        for entry in extensions {
            self.source_file_tree.retain(|x| !(x.path().is_file() &&
                x.path().to_str().expect("Error converting to &str in exclude_files_with_extension").ends_with(entry)));
        }
        match self.source_file_tree.is_empty() || self.source_file_tree.len() == start_len {
            true => Err("Source file tree hasn't changed or is empty"),
            false => Ok(())
        }
    }

    // TODO return Option
    pub fn copy(&mut self, to: &str) {
        if self.source_file_tree.is_empty() {
            println!("Error: folder tree is empty");
            std::process::exit(-1);
        }

        // First element of vector is always base folder path
        let base_input_path = self.source_file_tree[0].path().to_str().expect("Error converting source base path (self.source_file_tree[0].path()) to &str");
        let base_output_path = to;

        println!("Starting copying files...");
        // Copying
        for current_source in self.source_file_tree[1..].iter() {
            let current_source_path = current_source.path().to_str().expect("Error converting source path (&DirEntry.path()) to &str");
            let current_destination_path = current_source_path.replacen(base_input_path, base_output_path, 1);

            if current_source.path().is_dir() {
                if let Err(_) = fs::create_dir_all(&current_destination_path) {
                    println!("Error creating new directory: {}", &current_destination_path);
                }
            }
            else if current_source.path().is_file() {
                match fs::File::open(&current_source_path) {
                    Ok(mut source_file) => {
                        match fs::File::create(&current_destination_path) {
                            Ok(mut destination_file) => {
                                // io::copy was faster then fs::copy, because it's writing data "in streaming fashion"
                                if let Err(_) = io::copy(&mut source_file, &mut destination_file) {
                                    println!("Error copying to destination: {}", &current_destination_path);
                                    if let Err(_) = fs::remove_file(&current_destination_path) {
                                        println!("Error deleting empty file: {}", &current_destination_path);
                                    }
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