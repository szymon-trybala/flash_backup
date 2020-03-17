use walkdir::{WalkDir, DirEntry};
use std::fs;
use std::io;
use std::path::{MAIN_SEPARATOR};

pub struct Copying {
    pub source_files_tree: Vec<Vec<DirEntry>>,
    pub copied_entries: Vec<DirEntry>,
    pub full_output_path: String,
}

impl Copying {
    pub fn new(source_folders: &Vec<String>) -> Result<Copying, &'static str> {
        let mut copying = Copying { source_files_tree: Vec::new(), copied_entries: Vec::new(), full_output_path: String::new() };
        println!("Creating file maps...");

        for folder in source_folders {
            let mut map = Vec::new();
            for entry in WalkDir::new(&folder).into_iter().filter_map(|e| e.ok()) {
                map.push(entry);
            }
            // Skipping OneDrive temp file
            map.retain(|x| x.path().file_name().unwrap() != ".849C9593-D756-4E56-8D6E-42412F2A707B");
            if map.len() > 0 {
                copying.source_files_tree.push(map);
            }
        }
        println!("File maps created!");

        if copying.source_files_tree.len() == 0 {
            Err("all folders are empty or non existing")
        } else {
            Ok(copying)
        }
    }

    pub fn exclude_folders(&mut self, folders: &Vec<String>) -> Result<(), &'static str> {
        // TODO - ADD MULTITHREADING
        if self.source_files_tree.is_empty() {
            return Err("folder tree is empty");
        }
        println!("Starting excluding folders...");

        let mut folders_ignored: usize = 0;
        let mut files_ignored: usize = 0;

        for i in 0..self.source_files_tree.len() {
            let files_start: usize = self.source_files_tree[i].iter().filter(|x| x.path().is_file()).count();
            let folder_start: usize = self.source_files_tree[i].iter().filter(|x| x.path().is_dir()).count();

            for entry in folders {
                // Collecting folders to exclude
                let folders_to_exclude: Vec<DirEntry> = self.source_files_tree[i].iter().
                    filter(|x| x.path().exists() && x.path().is_dir()
                        && x.path().to_str().expect("Fatal error while collecting folders to exclude")
                        .contains(entry.as_str())).cloned().collect();

                // Excluding folders
                self.source_files_tree[i].retain(|x| !(x.path().exists() && x.path().is_dir()
                    && x.path().to_str().expect("Fatal error while excluding folders").contains(entry.as_str())));

                // Excluding files in excluded folders
                for folder in folders_to_exclude {
                    self.source_files_tree[i].retain(|x| !(x.path().starts_with(folder.path())));
                }
            }
            folders_ignored += folder_start - self.source_files_tree[i].iter().filter(|x| x.path().is_dir()).count();
            files_ignored += files_start - self.source_files_tree[i].iter().filter(|x| x.path().is_file()).count();
        }

        if folders_ignored == 0 {
            println!("No folders matching .ignore found");
        } else {
            println!("Ignored {} folders and {} files in them!", folders_ignored, files_ignored);
        }
        Ok(())
    }

    pub fn exclude_files_with_extensions(&mut self, extensions: &Vec<String>) -> Result<(), &'static str> {
        let mut unchanged_maps: usize = 0;
        let mut excluded_files: usize = 0;

        if self.source_files_tree.is_empty() {
            return Err("file tree is empty");
        }
        println!("Starting excluding files with selected extensions...");

        for i in 0..self.source_files_tree.len() {
            let start_len = self.source_files_tree[i].len();
            for entry in extensions {
                self.source_files_tree[i].retain(|x| !(x.path().is_file() &&
                    x.path().to_str().expect("couldn't filter path map").ends_with(entry)));
            }
            match self.source_files_tree[i].is_empty() || self.source_files_tree[i].len() == start_len {
                true => {
                    unchanged_maps += 1;
                }
                false => {
                    excluded_files += start_len - self.source_files_tree[i].len();
                }
            }
        }

        if unchanged_maps == self.source_files_tree.len() {
            println!("No files matching exclude found");
        } else {
            println!("Succesfully excluded {} files", excluded_files);
        }
        Ok(())
    }

    pub fn copy(&mut self, to: &str) -> Result<(), &'static str> {
        if self.source_files_tree.is_empty() {
            return Err("folder tree is empty");
        }
        let mut copied_count: usize = 0;

        println!("Starting copying files...");
        for i in 0..self.source_files_tree.len() {
            let current_folder_path = String::from(to) + MAIN_SEPARATOR.to_string().as_str() + self.source_files_tree[i][0].file_name().to_str().unwrap();
            println!("Copying from folder {}", &current_folder_path);
            if self.source_files_tree[i].is_empty() {
                println!("Folder subtree is empty, skipping");
                continue;
            }

            match fs::create_dir_all(&current_folder_path) {
                Err(_) => {
                    return Err("couldn't create folder to copy files");
                }
                Ok(_) => {
                    // First element of current map is always base folder path
                    let base_input_path_copied = String::from(self.source_files_tree[i][0].path().to_str().unwrap());
                    let base_input_path = base_input_path_copied.as_str();
                    let base_output_path = current_folder_path;

                    // Copying
                    for current_source in self.source_files_tree[i].iter() {
                        let current_source_path = String::from(current_source.path().to_str().expect("Fatal error converting source path (&DirEntry.path()) to &str"));
                        let current_destination_path = current_source_path.replacen(base_input_path, base_output_path.as_str(), 1);

                        if current_source.path().is_dir() {
                            if let Err(_) = fs::create_dir_all(&current_destination_path) {
                                println!("Error creating new directory: {}, skipping it and its content...", &current_destination_path);
                            }
                        } else if current_source.path().is_file() {
                            match fs::File::open(&current_source_path) {
                                Ok(mut source_file) => {
                                    match fs::File::create(&current_destination_path) {
                                        Ok(mut destination_file) => {
                                            println!("Copying file: {}", &current_source_path);
                                            match io::copy(&mut source_file, &mut destination_file) {
                                                Ok(_) => {
                                                    copied_count += 1;
                                                }
                                                Err(_) => {
                                                    println!("Error copying to destination: {}, skipping...", &current_destination_path);
                                                    if let Err(_) = fs::remove_file(&current_destination_path) {
                                                        println!("Error deleting empty file: {}", &current_destination_path);
                                                    }
                                                    // TODO - Deleting skipped file from vector, handle this in separate function!
                                                    // self.source_file_tree.retain(|x|
                                                    //     &x.path().to_str().expect("Error while converting path to string") != &current_source_path);
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            println!("Error creating file: {}", &current_destination_path);
                                            continue;
                                        }
                                    }
                                }
                                Err(_) => {
                                    println!("Error reading source file: {}", &current_source_path);
                                    continue;
                                }
                            }
                        }
                    }
                    // CREATING MAP OF COPIED FILES AND FOLDERS
                    for copied_entry in WalkDir::new(&base_output_path).into_iter().filter_map(|e| e.ok()) {
                        self.copied_entries.push(copied_entry);
                    }
                }
            }
            println!("Copied!");
        }
        println!("Copied {} files", copied_count);
        Ok(())
    }
}