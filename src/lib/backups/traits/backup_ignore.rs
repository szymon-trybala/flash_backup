use crate::backups::map::backup_dir::BackupDir;
use crate::backups::map::backup_entry::BackupEntry;
use std::sync::{Arc, Mutex};
use scoped_threadpool::Pool;
use std::borrow::BorrowMut;
use crate::backups::helpers::multithreading::arc_to_inner;

pub trait BackupIgnore {
    fn ignore_files_and_folders_parrarel(backup_dirs: Vec<BackupDir>, extensions_to_ignore: &Vec<String>, folders_to_ignore: &Vec<String>) -> Vec<BackupDir> {
        let mut backup_dirs = Arc::new(Mutex::new(backup_dirs));
        backup_dirs = ignore_extensions_parrarel(backup_dirs, extensions_to_ignore);
        backup_dirs = ignore_folders_parrarel(backup_dirs, folders_to_ignore);

        match arc_to_inner(backup_dirs) {
            Ok(dirs) => {
                if !dirs.is_empty() {
                    dirs
                } else {
                    panic!("Fatal error while trying to create input maps - all maps are empty");
                }
            }
            Err(e) => {
                let message = format!("Fatal error while trying to create input maps - {}", e);
                panic!(message);
            }
        }
    }
}

pub fn ignore_extensions_single_folder(folder: &mut BackupDir, extensions_to_ignore: &Vec<String>) -> Result<(), String> {
    // Checking input
    if folder.backup_entries.is_empty() {
        let message = format!("Folder {} is empty", &folder.root_input);
        return Err(message);
    }
    if extensions_to_ignore.is_empty() {
        return Err(String::from("No file ignores provided"));
    }

    // Ignoring extensions
    for extension in extensions_to_ignore {
        folder.backup_entries.retain(|x| !(x.is_file && x.input_path.ends_with(extension)));
    }

    Ok(())
}

pub fn ignore_folders_single_folder(folder: &mut BackupDir, folders_to_ignore: &Vec<String>) -> Result<(), String> {
    // Checking input
    if folder.backup_entries.is_empty() {
        let message = format!("Folder {} is empty", &folder.root_input);
        return Err(message);
    }
    if folders_to_ignore.is_empty() {
        return Err(String::from("No folder ignores provided"));
    }

    // Ignoring
    for folder_to_ignore in folders_to_ignore {
        let excluded_folders: Vec<BackupEntry> = folder.backup_entries.clone().into_iter().filter(|x| !x.is_file && x.input_path.contains(folder_to_ignore)).collect();
        folder.backup_entries.retain(|x| !(!x.is_file && x.input_path.contains(folder_to_ignore)));             // Ignoring folders
        for excluded_folder in excluded_folders {
            folder.backup_entries.retain(|x| !(x.input_path.starts_with(&excluded_folder.input_path)));         // Ignoring files in folders
        }
    }

    Ok(())
}

pub fn ignore_extensions_parrarel(dirs: Arc<Mutex<Vec<BackupDir>>>, extensions_to_ignore: &Vec<String>) -> Arc<Mutex<Vec<BackupDir>>> {
    // Checking input
    let dirs_ref = Arc::clone(&dirs);
    if dirs_ref.lock().unwrap().is_empty() {
        println!("No dirs to ignore from provided!");
        return dirs;
    }

    // Creating necessary variables
    println!("Removing ignored files from maps...");
    let len = dirs_ref.lock().unwrap().len();
    let max_threads = num_cpus::get();
    let mut thread_pool = Pool::new(max_threads as u32);

    // Creating threads and ignoring files from all of them
    thread_pool.scoped(|scoped| {
        for i in 0..len {
            let dirs_ref = Arc::clone(&dirs);

            scoped.execute(move || {
                let mut dirs_temp = dirs_ref.lock().unwrap();
                if let Err(e) = ignore_extensions_single_folder(dirs_temp[i].borrow_mut(), &extensions_to_ignore) {
                    println!("Error while ignoring files: {}", e);
                }
            });
        }
    });
    println!("Ignored files with provided extensions");
    dirs
}

pub fn ignore_folders_parrarel(dirs: Arc<Mutex<Vec<BackupDir>>>, folders_to_ignore: &Vec<String>) -> Arc<Mutex<Vec<BackupDir>>> {
    // Checking input
    let dirs_ref = Arc::clone(&dirs);
    if dirs_ref.lock().unwrap().is_empty() {
        println!("No dirs to ignore from provided!");
        return dirs;
    }

    // Creating necessary variables
    println!("Removing ignored folders from maps...");
    let len = dirs_ref.lock().unwrap().len();
    let max_threads = num_cpus::get();
    let mut thread_pool = Pool::new(max_threads as u32);

    // Creating threads and ignoring folders from all of them
    thread_pool.scoped(|scoped| {
        for i in 0..len {
            let dirs_ref = Arc::clone(&dirs);
            scoped.execute(move || {
                let mut dirs_temp = dirs_ref.lock().unwrap();

                if let Err(e) = ignore_folders_single_folder(dirs_temp[i].borrow_mut(), &folders_to_ignore) {
                    println!("Error while ignoring folders: {}", e);
                }
            });
        }
    });
    println!("Ignored provided folders from maps");
    dirs
}