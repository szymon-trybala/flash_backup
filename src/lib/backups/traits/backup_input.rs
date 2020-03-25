use crate::backups::map::backup_dir::BackupDir;
use std::path::{Path};
use walkdir::WalkDir;
use crate::backups::map::backup_entry::BackupEntry;
use crate::backups::helpers::hashing::generate_hash_meow_hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::borrow::BorrowMut;

#[cfg(test)]
#[test]
fn it_works() {
    // let dir = vec![BackupDir { root_input: String::from("home/szymon/Downloads/meowhash-rs"), folders: 0, files: 0, root_output: String::new(), backup_entries: Vec::new() }];
    let dir = check_input_folders(&vec![String::from("/home/szymon/Downloads/meowhash-rs")]);
    let mut backup_dirs = Arc::new(Mutex::new(dir));
    let dir = fill_backup_dirs_parallel(backup_dirs);
    let dir = Arc::try_unwrap(dir);
    let dir = dir.unwrap_or_default();
    let dir = dir.into_inner().unwrap_or_default();
}

pub trait BackupInput {
    // TODO - doc, test
    fn create_input_maps(paths: &Vec<String>) -> Result<Vec<BackupDir>, String> {
        let backup_dirs = check_input_folders(paths);
        let mut backup_dirs = Arc::new(Mutex::new(backup_dirs));
        backup_dirs = fill_backup_dirs_parallel(backup_dirs);
        // TODO - ugly, fix it
        let backup_dirs = Arc::try_unwrap(backup_dirs).unwrap_or_default().into_inner().unwrap_or_default();
        Ok(backup_dirs)
        // TODO - check for errors, return something
    }
}

/// Returns Vec<Backup_Dir> filled with Backup_Dir for every passed valid and non-empty path, only present field in Backup_Dir's is root_input.
/// Panics if there are no valid and non-empty input folders - there is no point of continuing the program if this happens
/// # Examples:
/// Replace "szymon" with your username. Test only for Linux.
// TODO - znaleźć jakiś plik na każdym linuksie i zrobić test tylko dla niego.
/// ```
/// let paths = vec![String::from("/home/szymon/Downloads")];
/// let result = check_input_folders(&paths);
/// assert_eq!(result[0].root_input, "home/szymon/Downloads");
fn check_input_folders(folders: &Vec<String>) -> Vec<BackupDir> {
    let mut dirs = Vec::new();

    for folder in folders {
        match Path::new(folder).exists() {
            true => {
                dirs.push(BackupDir {root_input: folder.clone(), root_output: String::new(), folders: 0, files: 0, backup_entries: Vec::new()})
            }
            false => {
                println!("Folder {} doesn't exist or is empty, skipping...", folder);
                continue;
            },
        }
    }

    match dirs.is_empty() {
        true => panic!("Fatal error: no existing and non-empty folder provided"),
        false => dirs
    }
}

/// Fills data in every BackupDir concurrently - one thread per one folder with maximum amount of present threads equal to amount of threads in computer.
/// Function handles creating threads, and then for every one of them it executes fill_single_backup_dir - for further info check its documentation.
/// Requires owned vector wrapped in Arc<Mutex>, returns filled vector, also as owned value. Vector is always returned.
/// If vector of BackupDir's is empty, it prints message and returns unchanged vector.
// TODO - add test
fn fill_backup_dirs_parallel(dirs: Arc<Mutex<Vec<BackupDir>>>) -> Arc<Mutex<Vec<BackupDir>>>  {
    // Checking input
    let dirs_ref = Arc::clone(&dirs);
    if dirs_ref.lock().unwrap().is_empty() {
        println!("All input dirs are empty!");
        return dirs;
    }

    // Creating threads and executing fill_single_backup_dir function on every of them
    let mut handles = vec![];
    let len = dirs_ref.lock().unwrap().len();
    // TODO - restrict threads amount (use thread count)
    for i in 0..len {
        let dirs_ref = Arc::clone(&dirs);
        let handle = thread::spawn(move || {
            let mut dirs_temp = dirs_ref.lock().unwrap();
            fill_single_backup_dir(dirs_temp[i].borrow_mut());
        });
        handles.push(handle);
    }
    for handle in handles {
        // TODO - co zrobić jak join nie zadziała :c
        handle.join();
    }
    dirs
}


/// Fills data of one BackupDir. FUNCTION ASSUMS THAT root_input IS ALREADY FILLED, if it isn't, it doesn't modify directory (and informs user about it)
/// Filled fields: folders, files, backup_entries (function creates Vec of BackupEntry, each one has input_path, is_file and hash filled).
///
/// Function may panic while converting DirEntry.path() to str, but it's almost impossible.
/// Function skip files for which hash couldn't be generated, user gets info about every skipped file.
// TODO - add test
fn fill_single_backup_dir(dir: &mut BackupDir) {
    // Checking input
    let as_path = Path::new(&dir.root_input);
    if dir.root_input.is_empty() || !as_path.exists() || as_path.is_file() {
        println!("Path {} doesn't exist or isn't a file", &dir.root_input);
        return;
    }

    // Creating map
    for entry in WalkDir::new(&dir.root_input).into_iter().skip(1).filter_map(|e| e.ok()) {
        match entry.path().is_file() {
            true => {
                let path = String::from(entry.path().to_str().expect("Unexpected error while creating maps"));
                match generate_hash_meow_hash(&path) {
                    Ok(hash) => {
                        dir.backup_entries.push(BackupEntry {input_path: path, output_path: String::new(), is_file: true, hash })
                    }
                    Err(e) => {
                        println!("{}, skipping...", e);
                        continue;
                    }
                }
            }
            false => {
                let path = String::from(entry.path().to_str().expect("Unexpected error while creating maps"));
                dir.backup_entries.push( BackupEntry {input_path: path, output_path: String::new(), is_file: false, hash: String::new()});
            }
        }
    }
    dir.files = dir.backup_entries.iter().filter(|x| x.is_file).count();
    dir.folders = dir.backup_entries.iter().filter(|x| !x.is_file).count();

    // Verifying that map was created
    if dir.backup_entries.is_empty() {
        println!("Couldn't find any entries in {}", &dir.root_input);
    } else {
        println!("Found {} files and {} folders in {}", dir.files, dir.folders, &dir.root_input)
    }
}
