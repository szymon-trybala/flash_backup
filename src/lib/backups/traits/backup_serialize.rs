use crate::backups::map::backup_map::BackupMap;
use uuid::Uuid;
use chrono::Utc;
use std::path::{Path, MAIN_SEPARATOR};
use crate::S_MAP;
use std::fs::{File, remove_file};
use std::io::{Write};
use crate::backups::map::backup_dir::BackupDir;
use crate::backups::helpers::hashing::generate_hash_meow_hash;
use std::sync::Arc;
use scoped_threadpool::Pool;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait BackupSerialize {
    fn serialize_to_json(map: &mut BackupMap) -> Result<(), String> {
        // Checking input
        let json_folder = Path::new(&map.output_folder);
        if !json_folder.exists() || json_folder.is_file() {
            return Err(String::from("Invalid output path"));
        }
        // Generating metadata, serializing
        println!("Saving folder map to JSON file...");
        generate_metadata(map);
        match serde_json::to_string_pretty(map) {
            Err(e) => {
                let message = format!("Can't convert data to text: {}", e);
                return Err(message);
            }
            Ok(json_string) => {
                let json_path = format!("{}{}{}", &map.output_folder, &MAIN_SEPARATOR.to_string(), S_MAP);
                match File::create(&json_path) {
                    Err(e) => {
                        let message = format!("Can't create JSON file {}: {}", json_path, e);
                        return Err(message);
                    }
                    Ok(mut json_file) => {
                        if let Err(e) = json_file.write_all(json_string.as_ref()) {
                            let mut message = format!("Can't copy serialized map to JSON file: {}", e);
                            if let Err(e) = remove_file(&json_path) {
                                message = format!("{} and can't remove JSON file: {}", message, e);
                            }
                            return Err(message);
                        }
                    }
                }
            }
        }
        println!("JSON file created");
        Ok(())
    }

    fn verify_all(map: &BackupMap) -> Result<(), String> {
        // Checking input
        if map.backup_dirs.is_empty() {
            return Err(String::from("No files to verify"));
        }

        // Verifying folders concurrently
        println!("Verifying copied files...");
        let corrupted = Arc::new(AtomicUsize::new(0));
        let map = Arc::new(map);
        let max_threads = num_cpus::get();
        let mut thread_pool = Pool::new(max_threads as u32);
        thread_pool.scoped(|scope| {
            for dir in &map.backup_dirs {
                let corrupted_ref = Arc::clone(&corrupted);
                scope.execute(move || {
                    match verify_one_folder(dir) {
                        Err(e) => {
                            println!("Can't verify: {}", e);
                        }
                        Ok(x) => {
                            corrupted_ref.fetch_add(x, Ordering::Relaxed);
                        }
                    }
                })
            }
        });

        // Displaying summary of verification
        match corrupted.load(Ordering::SeqCst) {
            0 => println!("Verification completed, all files are OK"),
            _ => println!("Verification completed, {} corrupted files", corrupted.load(Ordering::SeqCst))
        }
        Ok(())
    }
}

/// Returns error if there are no entries in BackupDir, or if all entries are corrupted (not one matching hash)
pub fn verify_one_folder(folder: &BackupDir) -> Result<usize, String> {
    // Checking input
    if folder.backup_entries.is_empty() {
        let message = format!("No entries in {} found", &folder.root_output);
        return Err(message);
    }

    // Comparing hashes
    let mut corrupted: usize = 0;
    for entry in &folder.backup_entries {
        if !entry.is_file {
            continue;
        }
        match generate_hash_meow_hash(&entry.output_path) {
            Err(e) => {
                println!("Can't generate hash of {} to verify integrity: {}", &entry.output_path, e);
                corrupted += 1;
                continue;
            }
            Ok(output_hash) => {
                if output_hash != entry.hash {
                    println!("Hashes of input {} and output {} files don't match", &entry.input_path, &entry.output_path);
                    corrupted += 1;
                    continue;
                }
            }
        }
    }

    // Checking amount of corrupted files
    match corrupted == folder.files {
        false => Ok(corrupted),
        true => {
            let message = format!("All files in {} are corrupted!", &folder.root_output);
            Err(message)
        }
    }
}

/// BackupMode has been already filled
pub fn generate_metadata(map: &mut BackupMap) {
    map.id = Uuid::new_v4().to_string();
    map.timestamp = Utc::now().timestamp() as usize;
    map.files = map.backup_dirs.iter().map(|x| x.files).sum();
    map.folders = map.backup_dirs.iter().map(|x| x.folders).sum();
}