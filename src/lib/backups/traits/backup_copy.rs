use crate::backups::map::backup_entry::BackupEntry;
use std::path::{Path};
use std::fs;
use std::io::{BufReader, BufWriter};
use crate::backups::map::backup_dir::BackupDir;
use std::sync::{Arc, Mutex};
use scoped_threadpool::Pool;
use std::borrow::BorrowMut;
use crate::backups::helpers::multithreading::arc_to_inner;
use crate::backups::helpers::dirs::get_last_subdir;

#[cfg(test)]
mod tests {
    use crate::backups::map::backup_dir::BackupDir;
    use crate::backups::map::backup_entry::BackupEntry;
    use crate::backups::modes::backup_cloud::BackupCloud;
    use crate::backups::traits::backup_copy::{BackupCopy, copy_folder, copy_file, create_folder, create_parent_folder};
    use std::path::Path;

    #[test]
    fn test_copy_all() {
        let mut backup_dirs = vec![BackupDir::new(), BackupDir::new()];
        backup_dirs[0].root_output = String::from("/home/szymon/Downloads/backup/1");
        backup_dirs[1].root_output = String::from("/home/szymon/Downloads/backup/2");
        let entry1 = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/home/szymon/Downloads/backup/1/bookmarks.html"), is_file: true, hash: String::new()};
        let entry2 = BackupEntry {input_path: String::from("/usr/bin/bash"), output_path: String::from("/home/szymon/Downloads/backup/2/bash"), is_file: true, hash: String::new()};
        backup_dirs[0].backup_entries.push(entry1);
        backup_dirs[1].backup_entries.push(entry2);
        backup_dirs = BackupCloud::copy_all(backup_dirs);
        assert!(Path::new(&backup_dirs[0].backup_entries[0].output_path).exists());
        assert!(Path::new(&backup_dirs[1].backup_entries[0].output_path).exists());
    }

    #[test]
    fn test_copy_folder() {
        let mut dir = BackupDir::new();
        dir.root_output = String::from("/home/szymon/Downloads/backup/1");
        let entry1 = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/home/szymon/Downloads/backup/1/bookmarks.html"), is_file: true, hash: String::new()};
        dir.backup_entries.push(entry1);
        copy_folder(&mut dir).unwrap();
        assert!(Path::new(&dir.backup_entries[0].output_path).exists());
    }

    #[test]
    fn test_copy_file() {
        let entry = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/home/szymon/Downloads/backup/1/bookmarks.html"), is_file: true, hash: String::new()};
        copy_file(&entry).unwrap();
        assert!(Path::new(&entry.output_path).exists());
    }

    #[test]
    fn test_create_folder() {
        let path = String::from("/home/szymon/Downloads/backup/new_path");
        create_folder(&path).unwrap();
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn test_create_parent_folder() {
        let path = String::from("/home/szymon/Downloads/completely_new/path.txt");
        create_parent_folder(&path).unwrap();
        assert!(Path::new("/home/szymon/Downloads/completely_new").exists());
    }
}

/// Provides function to copy all entries from input location to output location.
///
/// Requires completely filled BackupMap, after all processing.
pub trait BackupCopy {
    /// Copies all files from its input locations to output locations.
    ///
    /// Requires completely filled BackupMap, after all processing.
    ///
    /// Works concurrently, using max 2 threads at once - too many copying operations working at once can slow down process.
    ///
    /// Function may panic if fatal error occurs during multithreading operations and conversions - it's too dangerous to continue runtime at this point.
    ///
    /// If any file or folder can't be copied, user gets message, and this file is deleted from map, because serialized map needs to contain only copied files.
    ///
    /// # Example:
    /// This test requires usage of struct that implements BackupInput trait, like BackupCloud or BackupMultiple.
    /// To pass test you need to provide your own paths.
    /// ```
    /// use flash_backup::backups::map::backup_dir::BackupDir;
    /// use flash_backup::backups::map::backup_entry::BackupEntry;
    /// use flash_backup::backups::modes::backup_cloud::BackupCloud;
    /// use std::path::Path;
    /// use flash_backup::backups::traits::backup_copy::BackupCopy;
    /// let mut backup_dirs = vec![BackupDir::new(), BackupDir::new()];
    /// backup_dirs[0].root_output = String::from("/home/szymon/Downloads/backup/1");
    /// backup_dirs[1].root_output = String::from("/home/szymon/Downloads/backup/2");
    /// let entry1 = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/home/szymon/Downloads/backup/1/bookmarks.html"), is_file: true, hash: String::new()};
    /// let entry2 = BackupEntry {input_path: String::from("/usr/bin/bash"), output_path: String::from("/home/szymon/Downloads/backup/2/bash"), is_file: true, hash: String::new()};
    /// backup_dirs[0].backup_entries.push(entry1);
    /// backup_dirs[1].backup_entries.push(entry2);
    /// backup_dirs = BackupCloud::copy_all(backup_dirs);
    /// assert!(Path::new(&backup_dirs[0].backup_entries[0].output_path).exists());
    /// assert!(Path::new(&backup_dirs[1].backup_entries[0].output_path).exists());
    /// ```
    fn copy_all(dirs: Vec<BackupDir>) -> Vec<BackupDir> {
        // Checking input
        if dirs.is_empty() {
            println!("No dirs to copy provided");
            return dirs;
        }
        println!("Copying files...");
        // Creating necessary variables
        let len = dirs.len();
        let dirs = Arc::new(Mutex::new(dirs));
        let mut thread_pool = Pool::new(2);

        // Copying - one thread per one BackupDir
        thread_pool.scoped(|scoped| {
            for i in 0..len {
                let dirs_ref = Arc::clone(&dirs);
                scoped.execute(move || {
                    let mut dirs_temp = dirs_ref.lock().unwrap();
                    if let Err(e) = copy_folder(&mut dirs_temp[i].borrow_mut()) {
                        println!("Error while copying: {}", e);
                    }
                });
            }
        });

        // Converting to inner value
        match arc_to_inner(dirs) {
            Ok(dirs) => dirs,
            Err(e) => {
                let message = format!("Fatal error while trying to create input maps - {}. Program will stop", e);
                panic!(message);
            }
        }
    }
}

/// Copies content of one BackupDir - for each entry, from input path to output path.
///
/// Requires completely filled BackupDir, after all processing.
///
/// Function returns error only if no one entry could be copied. Every other error is printed to user and doesn't stop function execution.
///
/// BackupDir is modified - every **NOT COPIED** element is removed from it.
///
/// # Example:
/// To pass test you need to provide your own paths.
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use flash_backup::backups::map::backup_entry::BackupEntry;
/// use std::path::Path;
/// use flash_backup::backups::traits::backup_copy::copy_folder;
/// let mut dir = BackupDir::new();
/// dir.root_output = String::from("/home/szymon/Downloads/backup/1");
/// let entry1 = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/home/szymon/Downloads/backup/1/bookmarks.html"), is_file: true, hash: String::new()};
/// dir.backup_entries.push(entry1);
/// copy_folder(&mut dir).unwrap();
/// assert!(Path::new(&dir.backup_entries[0].output_path).exists());
/// ```
pub fn copy_folder(folder: &mut BackupDir) -> Result<(), String> {
    // Creating root folder
    if let Err(e) = create_folder(&folder.root_output) {
        let message = format!("Couldn't copy folder {}: can't create root folder: {}", &folder.root_input, e);
        return Err(message);
    }

    println!("Copying folder {}...", &folder.root_input);
    let mut copied_entries = vec![];
    let mut not_filled_entries: usize = 0;

    for entry in &folder.backup_entries {
        if entry.output_path.is_empty() || entry.input_path.is_empty() {
            not_filled_entries += 1;
            continue;
        }

        match entry.is_file {
            true => {
                match copy_file(&entry) {
                    Ok(_) => copied_entries.push(entry.clone()),
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                }
            }
            false => {
                match create_folder(&entry.output_path) {
                    Ok(_) => copied_entries.push(entry.clone()),
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                }
            }
        }
    }
    if not_filled_entries > 0 {
        println!("{} entries needs more data to copy", not_filled_entries);
    }

    match copied_entries.is_empty() {
        true => {
            let message = format!("No entries copied from {}", &folder.root_input);
            return Err(message);
        }
        false => {
            let not_copied_entries = folder.backup_entries.len() - copied_entries.len();
            if not_copied_entries > 0 {
                println!("Couldn't copy {} entries from {}", not_copied_entries, &folder.root_input);
            }
            folder.backup_entries = copied_entries;
            folder.files = folder.backup_entries.iter().filter(|x| x.is_file).count();
            folder.folders = folder.backup_entries.iter().filter(|x| !x.is_file).count();
            println!("Copied {} folders and {} files from folder {}", folder.folders, folder.files, &folder.root_input);
            Ok(())
        }
    }
}

/// Copies file from input path to output path (in BackupEntry).
///
/// Requires completely filled BackupEntry, after all processing.
///
/// Returns error if BackupEntry is not filled, if parent directory can't be created (if needed), if input file can't be opened, if destination file can't be created, or if copying can't be done.
///
/// # Example:
/// To pass test you need to provide your own paths.
/// ```
/// use flash_backup::backups::map::backup_entry::BackupEntry;
/// use flash_backup::backups::traits::backup_copy::copy_file;
/// use std::path::Path;
/// let entry = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("/home/szymon/Downloads/backup/1/bookmarks.html"), is_file: true, hash: String::new()};
/// copy_file(&entry).unwrap();
/// assert!(Path::new(&entry.output_path).exists());
/// ```
pub fn copy_file(entry: &BackupEntry) -> Result<(), String> {
    // Checking input
    if entry.input_path.is_empty() || entry.output_path.is_empty() {
        return Err(String::from("Entry not filled"));
    }

    // Creating parent folder if it doesn't exist
    if let Err(e) = create_parent_folder(&entry.output_path) {
        let message = format!("File {} not copied: {}", &entry.input_path, e);
        return Err(message);
    }

    // Creating opening files, creating reader and writer
    match fs::File::open(&entry.input_path) {
        Err(e) => {
            let message = format!("Couldn't copy file {} to destination {}: can't open source file: {}", &entry.input_path, &entry.output_path, e);
            return Err(message);
        }
        Ok(source) => {
            let mut buf_reader = BufReader::new(source);
            match fs::File::create(&entry.output_path) {
                Err(e) => {
                    let message = format!("Couldn't copy file {} to destination {}: can't create destnation file: {}", &entry.input_path, &entry.output_path, e);
                    return Err(message);
                }
                Ok(destination) => {
                    // Copying file
                    let mut buf_writer = BufWriter::new(destination);
                    if let Err(e) = std::io::copy(&mut buf_reader, &mut buf_writer) {
                        let message = format!("Couldn't copy file {} to destination {}: {}", &entry.input_path, &entry.output_path, e);
                        return Err(message);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Creates folder with all its parent folders (if they doesn't exist).
///
/// May return error if path is empty or folder can't be created.
///
/// # Example:
/// To pass test you need to provide your own paths.
/// ```
/// use flash_backup::backups::traits::backup_copy::create_folder;
/// use std::path::Path;
/// let path = String::from("/home/szymon/Downloads/backup/new_path");
/// create_folder(&path).unwrap();
/// assert!(Path::new(&path).exists());
/// ```
pub fn create_folder(folder: &String) -> Result<(), String> {
    // Checking input
    if folder.is_empty() {
        return Err(String::from("Can't create folder - path not provided"));
    }
    // Creating folder
    if !Path::new(folder).exists() {
        if let Err(e) = fs::create_dir_all(folder) {
            let message = format!("Couldn't create folder: {}: {}", &folder, e);
            return Err(message);
        }
    }
    Ok(())
}

/// Creates parent folder for provided path.
///
/// Returns error if path is empty, if getting last folder fails, or creating folder fails.
///
/// # Example:
/// ```
/// use flash_backup::backups::traits::backup_copy::{create_folder, create_parent_folder};
/// use std::path::Path;
/// let path = String::from("/home/szymon/Downloads/completely_new/path.txt");
/// create_parent_folder(&path).unwrap();
/// assert!(Path::new("/home/szymon/Downloads/completely_new").exists());
/// ```
pub fn create_parent_folder(file_path: &String) -> Result<(), String> {
    // Checking input
    if file_path.is_empty() {
        return Err(String::from("Can't create parent folder - path not provided"));
    }

    // Extracting parent folder
    match get_last_subdir(file_path) {
        Err(e) => {
            let message = format!("Couldn't create parent folder to file {}: {}", file_path, e);
            return Err(message);
        }
        Ok(file_path_last) => {
            let file_parent_folder = file_path.trim_end_matches(&file_path_last);
            // Creating folder if it doesn't exist
            if !Path::new(file_parent_folder).exists() {
                if let Err(e) = fs::create_dir_all(file_parent_folder) {
                    let message = format!("Couldn't create folder: {} to copy file: {}: {}", &file_parent_folder, file_path, e);
                    return Err(message);
                }
            }
        }
    }
    Ok(())
}