use crate::backups::map::backup_map::BackupMap;
use std::path::{Path, MAIN_SEPARATOR};
use crate::S_MAP;
use std::fs::{File, remove_file};
use std::io::{Write};
use crate::backups::map::backup_dir::BackupDir;
use crate::backups::helpers::hashing::generate_hash_meow_hash;
use std::sync::{Arc, Mutex};
use scoped_threadpool::Pool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::BorrowMut;
use crate::backups::helpers::multithreading::arc_to_inner;

#[cfg(test)]
mod tests {
    use crate::backups::map::backup_mode::BackupMode;
    use crate::backups::map::backup_map::BackupMap;
    use crate::backups::modes::backup_multiple::BackupMultiple;
    use crate::backups::traits::backup_serialize::{BackupSerialize, verify_one_folder};
    use std::path::Path;
    use crate::backups::map::backup_dir::BackupDir;
    use crate::backups::map::backup_entry::BackupEntry;
    use crate::backups::helpers::hashing::generate_hash_meow_hash;

    #[test]
    fn test_serialize_to_json() {
        let mut map = BackupMap::new(BackupMode::Multiple);
        map.output_folder = String::from("/home/szymon/Downloads/");
        BackupMultiple::serialize_to_json(&mut map);
        assert!(Path::new("/home/szymon/Downloads/.map.json").exists());
    }

    #[test]
    fn test_verify_all() {
        let mut map = BackupMap::new(BackupMode::Multiple);
        let mut backup_dirs = vec![BackupDir::new(), BackupDir::new()];
        backup_dirs[0].root_output = String::from("/home/szymon/Downloads/backup/1");
        backup_dirs[1].root_output = String::from("/home/szymon/Downloads/backup/2");
        let entry1 = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/usr/lib/chromium/bookmarks.html"), is_file: true, hash: generate_hash_meow_hash("/usr/lib/chromium/bookmarks.html").unwrap()};
        let entry2 = BackupEntry {input_path: String::from("/usr/bin/bash"), output_path: String::from("/usr/bin/bash"), is_file: true, hash: generate_hash_meow_hash("/usr/bin/bash").unwrap()};
        backup_dirs[0].backup_entries.push(entry1);
        backup_dirs[1].backup_entries.push(entry2);
        map.backup_dirs = backup_dirs;
        let result = BackupMultiple::verify_all(&map).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_verify_one_folder() {
        let mut dir = BackupDir::new();
        dir.files = 1;
        dir.backup_entries.push(BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/usr/lib/chromium/bookmarks.html"), is_file: true, hash: generate_hash_meow_hash("/usr/lib/chromium/bookmarks.html").unwrap()});
        let result = verify_one_folder(&dir).unwrap();
        assert_eq!(result, 0);
    }
}

/// Provides functions to serialize BackupMap to JSON file, and check file integrity.
///
/// Should be used with completely filled BackupMap, after all processing.
pub trait BackupSerialize {
    fn delete_non_existing(dirs: Vec<BackupDir>) -> Vec<BackupDir> {
        let len = dirs.len();
        let dirs = Arc::new(Mutex::new(dirs));
        let max_threads = num_cpus::get();
        let mut thread_pool = Pool::new(max_threads as u32);

        thread_pool.scoped(|scope| {
            for i in 0..len {
                let dirs_ref = Arc::clone(&dirs);

                scope.execute(move || {
                    let mut dirs_temp = dirs_ref.lock().unwrap();
                    delete_non_existing_one_folder(dirs_temp[i].borrow_mut());
                });
            }
        });

        match arc_to_inner(dirs) {
            Ok(dirs) => {
                if dirs.is_empty() {
                    println!("Error while trying to track non-existing files - all maps are empty");
                }
                dirs
            }
            Err(e) => {
                let message = format!("Fatal error while trying to create input maps - {}. Program will stop", e);
                panic!(message);
            }
        }
    }
    /// Serializes map to JSON file.
    ///
    /// Should be used with completely filled BackupMap, after all processing, requires filled path to main output folder.
    ///
    /// May return error if required data (map output folder) isn't filled, if map can't be converted to text, if file can't be created, or if data can't be saved to file.
    ///
    /// # Example:
    /// This test requires usage of struct that implements BackupInput trait, like BackupCloud or BackupMultiple.
    /// To pass test you need to provide your own paths and ignores variables.
    /// ```
    /// use flash_backup::backups::map::backup_map::BackupMap;
    /// use flash_backup::backups::map::backup_mode::BackupMode;
    /// use flash_backup::backups::modes::backup_multiple::BackupMultiple;
    /// use std::path::Path;
    /// use flash_backup::backups::traits::backup_serialize::BackupSerialize;
    /// let mut map = BackupMap::new(BackupMode::Multiple);
    /// map.output_folder = String::from("/home/szymon/Downloads/");
    /// BackupMultiple::serialize_to_json(&mut map);
    /// assert!(Path::new("/home/szymon/Downloads/.map.json").exists());
    /// ```
    fn serialize_to_json(map: &mut BackupMap) -> Result<(), String> {
        // Checking input
        let json_folder = Path::new(&map.output_folder);
        if !json_folder.exists() || json_folder.is_file() {
            return Err(String::from("Invalid output path"));
        }

        // Generating metadata, serializing
        println!("Saving folder map to JSON file...");
        map.generate_metadata();
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

    /// Verifies integrity of all copied files.
    ///
    /// Intended to be used after copying of files and with completely filled and processed BackupMap.
    ///
    /// Works concurrently, max amount of active threads is 4, because reading from too many hard drive's locations at once can slow down process.
    ///
    /// Function may return error if needed data is not filled. In any other case function returns Ok with number of corrupted files, every corrupted file is reported to user.
    ///
    /// # Example
    /// To pass test you need to provide your own paths.
    /// ```
    /// use flash_backup::backups::map::backup_map::BackupMap;
    /// use flash_backup::backups::map::backup_mode::BackupMode;
    /// use flash_backup::backups::map::backup_dir::BackupDir;
    /// use flash_backup::backups::map::backup_entry::BackupEntry;
    /// use flash_backup::backups::helpers::hashing::generate_hash_meow_hash;
    /// use flash_backup::backups::modes::backup_multiple::BackupMultiple;
    /// use flash_backup::backups::traits::backup_serialize::BackupSerialize;
    /// let mut map = BackupMap::new(BackupMode::Multiple);
    /// let mut backup_dirs = vec![BackupDir::new(), BackupDir::new()];
    /// backup_dirs[0].root_output = String::from("/home/szymon/Downloads/backup/1");
    /// backup_dirs[1].root_output = String::from("/home/szymon/Downloads/backup/2");
    /// let entry1 = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/usr/lib/chromium/bookmarks.html"), is_file: true, hash: generate_hash_meow_hash("/usr/lib/chromium/bookmarks.html").unwrap()};
    /// let entry2 = BackupEntry {input_path: String::from("/usr/bin/bash"), output_path: String::from("/usr/bin/bash"), is_file: true, hash: generate_hash_meow_hash("/usr/bin/bash").unwrap()};
    /// backup_dirs[0].backup_entries.push(entry1);
    /// backup_dirs[1].backup_entries.push(entry2);
    /// map.backup_dirs = backup_dirs;
    /// let result = BackupMultiple::verify_all(&map).unwrap();
    /// assert_eq!(result, 0);
    /// ```
    fn verify_all(map: &BackupMap) -> Result<usize, String> {
        // Checking input
        if map.backup_dirs.is_empty() {
            return Err(String::from("No files to verify"));
        }

        // Verifying folders concurrently
        println!("Verifying copied files...");
        let corrupted = Arc::new(AtomicUsize::new(0));
        let map = Arc::new(map);
        let mut thread_pool = Pool::new(4);
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
        Ok(corrupted.load(Ordering::SeqCst))
    }
}

/// Verifies integrity of files in one directory.
///
/// Intended to be used after copying of files and with completely filled and processed BackupMap.
///
///  Returns error if there are no entries in BackupDir, or if all entries are corrupted (not one matching hash found). In other case, returns number of corrupted files in this folder.
///
/// # Example:
/// To pass test you need to provide your own paths.
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use flash_backup::backups::traits::backup_serialize::verify_one_folder;
/// use flash_backup::backups::helpers::hashing::generate_hash_meow_hash;
/// use flash_backup::backups::map::backup_entry::BackupEntry;
/// let mut dir = BackupDir::new();
/// dir.files = 1;
/// dir.backup_entries.push(BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/usr/lib/chromium/bookmarks.html"), is_file: true, hash: generate_hash_meow_hash("/usr/lib/chromium/bookmarks.html").unwrap()});
/// let result = verify_one_folder(&dir).unwrap();
/// assert_eq!(result, 0);
/// ```
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
        false => {
            if corrupted > 0 {
                println!("{} files in {} are corrupted", corrupted, &folder.root_output);
            }
            Ok(corrupted)
        },
        true => {
            let message = format!("All files in {} are corrupted!", &folder.root_output);
            Err(message)
        }
    }
}

/// Deletes from BackupDir all entries whose output path doesn't exist.
///
/// Should be used straight before verification.
pub fn delete_non_existing_one_folder(folder: &mut BackupDir) {
    let mut verified = vec![];
    for entry in &folder.backup_entries {
        if Path::new(&entry.output_path).exists() {
            verified.push(entry.clone());
        }
    }
    folder.backup_entries = verified;
}