use crate::backups::map::backup_entry::BackupEntry;
use std::path::{MAIN_SEPARATOR, Path};
use std::fs;
use std::io::{BufReader, BufWriter};
use crate::backups::map::backup_dir::BackupDir;
use std::sync::{Arc, Mutex};
use scoped_threadpool::Pool;
use std::borrow::BorrowMut;
use crate::backups::helpers::multithreading::arc_to_inner;

#[cfg(test)]
mod tests {
    use crate::backups::map::backup_dir::BackupDir;
    use crate::backups::map::backup_entry::BackupEntry;
    use crate::backups::modes::backup_cloud::BackupCloud;
    use crate::backups::traits::backup_copy::BackupCopy;

    #[test]
    fn test_copy_all() {
        let mut backup_dirs = vec![BackupDir::new(), BackupDir::new()];
        let entry1 = BackupEntry {input_path: String::from("/usr/lib/chromium/bookmarks.html"), output_path: String::from("home/szymon/Downloads/bookmarks.html"), is_file: true, hash: String::new()};
        let entry2 = BackupEntry {input_path: String::from("/usr/bin/bash"), output_path: String::from("home/szymon/Downloads/bash"), is_file: true, hash: String::new()};
        backup_dirs[0].backup_entries.push(entry1);
        backup_dirs[1].backup_entries.push(entry2);
        backup_dirs = BackupCloud::copy_all(backup_dirs);
    }
}

pub trait BackupCopy {
    fn copy_all(dirs: Vec<BackupDir>) -> Vec<BackupDir> {
        // Checking input
        if dirs.is_empty() {
            println!("No dirs to copy provided");
            return dirs;
        }
        // Creating necessary variables
        let len = dirs.len();
        let dirs = Arc::new(Mutex::new(dirs));
        let max_threads = num_cpus::get();
        let mut thread_pool = Pool::new(max_threads as u32);

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
                let message = format!("Fatal error while trying to create input maps - {}", e);
                panic!(message);
            }
        }
    }
}

pub fn copy_folder(folder: &mut BackupDir) -> Result<(), String> {
    // Creating root folder
    if let Err(e) = create_folder(&folder.root_output) {
        let message = format!("Couldn't copy folder {}: can't create root folder: {}", &folder.root_input, e);
        return Err(message);
    }

    let mut copied_entries = vec![];

    for entry in &folder.backup_entries {
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
            Ok(())
        }
    }
}

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

pub fn create_parent_folder(file_path: &String) -> Result<(), String> {
    // Checking input
    if file_path.is_empty() {
        return Err(String::from("Can't create parent folder - path not provided"));
    }

    // Extracting parent folder
    let file_path_splitted: Vec<&str> = file_path.as_str().split(MAIN_SEPARATOR).collect();
    match file_path_splitted.last() {
        None => {
            let message = format!("Couldn't create parent folder to file: {}", file_path);
            return Err(message);
        }
        Some(file_path_last) => {
            let file_parent_folder = file_path.trim_end_matches(file_path_last);
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