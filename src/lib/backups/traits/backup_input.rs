use crate::backups::map::backup_dir::BackupDir;
use std::path::Path;
use walkdir::WalkDir;
use crate::backups::map::backup_entry::BackupEntry;
use crate::backups::helpers::hashing::generate_hash_meow_hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::borrow::BorrowMut;
use scoped_threadpool::Pool;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_input_folders() {
        let paths = vec![String::from("/usr/include/bash")];
        let result = check_input_folders(&paths);
        assert_eq!(result[0].root_input, "/usr/include/bash");
    }

    #[test]
    fn test_fill_single_backup_dir() {
        let mut dir = BackupDir { root_input: String::from("/usr/include/bash"), root_output: String::new(), files: 0, folders: 0, backup_entries: vec![] };
        fill_single_backup_dir(&mut dir);
        assert_eq!(dir.root_input, "/usr/include/bash");
        assert_eq!(dir.files, 58);
        assert_eq!(dir.folders, 2);
    }

    #[test]
    fn test_fill_backup_dirs_parallel_and_arc_to_inner() {
        let dirs = vec![BackupDir { root_input: String::from("/usr/include/bash"), root_output: String::new(), files: 0, folders: 0, backup_entries: vec![] }];
        let backup_dirs = Arc::new(Mutex::new(dirs));
        let backup_dirs = fill_backup_dirs_parallel(backup_dirs);
        let backup_dirs = arc_to_inner(backup_dirs).unwrap();
        assert_eq!(backup_dirs[0].root_input, "/usr/include/bash");
        assert_eq!(backup_dirs[0].files, 58);
        assert_eq!(backup_dirs[0].folders, 2);
    }

    #[test]
    fn test_all() {
        let paths = vec![String::from("/usr/bin/core_perl"), String::from("/usr/share/alsa"), String::from("/usr/share/gtk-doc"), String::from("/usr/share/help"), String::from("/usr/lib32/pulseaudio")];
        let dirs = check_input_folders(&paths);
        let dirs = Arc::new(Mutex::new(dirs));
        let dirs = fill_backup_dirs_parallel(dirs);
        let dirs = arc_to_inner(dirs).unwrap();
        assert_eq!(dirs.len(), 5);
        println!("EHS {}", dirs[1].files);
    }
}

/// Provides function to create and fill folder maps with all needed input data
pub trait BackupInput {
    /// Creates file map as Vec<BackupDir> based on input paths, ensures that no BackupDir is empty. Creating maps and generating hashes is done parallelly.
    ///
    /// Function may panic in few cases: if all input paths are invalid, if maps are empty, or if fatal error occurs during multithreading operations and conversions. If error occurs while processing/hashing some file,
    /// user gets message and file is skipped.
    ///
    /// Example (only for Linux, test may fail if your /usr/include/bash is different):
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use flash_backup::backups::traits::backup_input::{check_input_folders, fill_backup_dirs_parallel, arc_to_inner};
    /// let dir = check_input_folders(&vec![String::from("/usr/include/bash")]);
    /// let mut backup_dirs = Arc::new(Mutex::new(dir));
    /// let dir = fill_backup_dirs_parallel(backup_dirs);
    /// let dir = arc_to_inner(dir).unwrap();
    /// assert_eq!(dir[0].root_input, "/usr/include/bash");
    /// assert_eq!(dir[0].files, 58);
    /// assert_eq!(dir[0].folders, 2);
    /// ```
    fn create_input_maps(paths: &Vec<String>) -> Vec<BackupDir> {
        let backup_dirs = check_input_folders(paths);
        let mut backup_dirs = Arc::new(Mutex::new(backup_dirs));
        backup_dirs = fill_backup_dirs_parallel(backup_dirs);
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

/// Returns Vec<Backup_Dir> filled with Backup_Dir for every passed valid and non-empty path, only present field in Backup_Dir's is root_input.
///
/// Panics if there are no valid and non-empty input folders - there is no point of continuing the program if this happens
/// # Example (works only on Linux, test may fail if your system doesn't have /usr/include/bash):
/// ```
/// use flash_backup::backups::traits::backup_input::check_input_folders;
///
/// let paths = vec![String::from("/usr/include/bash")];
/// let result = check_input_folders(&paths);
/// assert_eq!(result[0].root_input, "/usr/include/bash");
/// ```
pub fn check_input_folders(folders: &Vec<String>) -> Vec<BackupDir> {
    let mut dirs = Vec::new();

    for folder in folders {
        match Path::new(folder).exists() {
            true => {
                dirs.push(BackupDir { root_input: folder.clone(), root_output: String::new(), folders: 0, files: 0, backup_entries: Vec::new() })
            }
            false => {
                println!("Folder {} doesn't exist or is empty, skipping...", folder);
                continue;
            }
        }
    }

    match dirs.is_empty() {
        true => panic!("Fatal error: no existing and non-empty folder provided"),
        false => dirs
    }
}

/// Fills data in every BackupDir concurrently - one thread per one folder with maximum amount of present threads equal to amount of threads in computer.
///
/// Function handles creating threads, and then for every one of them it executes fill_single_backup_dir - for further info check its documentation.
///
/// Requires owned vector wrapped in Arc<Mutex>, returns filled vector, also as owned value. Vector is always returned.
///
/// If vector of BackupDir's is empty, it prints message and returns unchanged vector.
/// # Example (works only on Linux, test may fail if your /usr/include/bash is different):
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use std::sync::{Arc, Mutex};
/// use flash_backup::backups::traits::backup_input::fill_backup_dirs_parallel;
///
/// let dirs = vec![BackupDir { root_input: String::from("/usr/include/bash"), root_output: String::new(), files: 0, folders: 0, backup_entries: vec![] }];
/// let backup_dirs = Arc::new(Mutex::new(dirs));
/// let backup_dirs = fill_backup_dirs_parallel(backup_dirs);
/// let backup_dirs = Arc::try_unwrap(backup_dirs).unwrap_or_default().into_inner().unwrap_or_default();
/// assert_eq!(backup_dirs[0].root_input, "/usr/include/bash");
/// assert_eq!(backup_dirs[0].files, 58);
/// assert_eq!(backup_dirs[0].folders, 2);
/// ```
pub fn fill_backup_dirs_parallel(dirs: Arc<Mutex<Vec<BackupDir>>>) -> Arc<Mutex<Vec<BackupDir>>> {
    // Checking input
    let dirs_ref = Arc::clone(&dirs);
    if dirs_ref.lock().unwrap().is_empty() {
        println!("All input dirs are empty!");
        return dirs;
    }

    // Creating threads and executing fill_single_backup_dir function on every of them
    println!("Creating maps");
    let len = dirs_ref.lock().unwrap().len();
    let max_threads = num_cpus::get();
    let mut thread_pool = Pool::new(max_threads as u32);
    thread_pool.scoped(|scoped| {
        for i in 0..len {
            let dirs_ref = Arc::clone(&dirs);
            scoped.execute(move || {
                let mut dirs_temp = dirs_ref.lock().unwrap();
                fill_single_backup_dir(dirs_temp[i].borrow_mut());
            });
        }
    });
    dirs
}

/// Fills data of one BackupDir.
///
/// **FUNCTION ASSUMS THAT root_input IS ALREADY FILLED, if it isn't, it doesn't modify directory (and informs user about it)**
/// Filled fields: folders, files, backup_entries (function creates Vec of BackupEntry, each one has input_path, is_file and hash filled).
///
/// Function may panic while converting DirEntry.path() to str, but it's almost impossible.
/// Function skip files for which hash couldn't be generated, user gets info about every skipped file.
/// # Example (works only on Linux, test may fail if your /usr/include/bash is different):
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use flash_backup::backups::traits::backup_input::fill_single_backup_dir;
/// let mut dir = BackupDir {root_input: String::from("/usr/include/bash"), root_output: String::new(), files: 0, folders: 0, backup_entries: vec![]};
/// fill_single_backup_dir(&mut dir);
/// assert_eq!(dir.root_input, "/usr/include/bash");
/// assert_eq!(dir.files, 58);
/// assert_eq!(dir.folders, 2);
/// ```
pub fn fill_single_backup_dir(dir: &mut BackupDir) {
    // Checking input
    let as_path = Path::new(&dir.root_input);
    if dir.root_input.is_empty() || !as_path.exists() || as_path.is_file() {
        println!("Path {} doesn't exist or isn't a file", &dir.root_input);
        return;
    }

    // Creating map
    for entry in WalkDir::new(&dir.root_input).into_iter().skip(1).filter_map(|e| e.ok()) { // skip(1) because first value is always root input
        match entry.path().is_file() {
            true => {
                let path = String::from(entry.path().to_str().expect("Unexpected error while creating maps"));
                match generate_hash_meow_hash(&path) {
                    Ok(hash) => {
                        dir.backup_entries.push(BackupEntry { input_path: path, output_path: String::new(), is_file: true, hash })
                    }
                    Err(e) => {
                        println!("{}, skipping...", e);
                        continue;
                    }
                }
            }
            false => {
                let path = String::from(entry.path().to_str().expect("Unexpected error while creating maps"));
                dir.backup_entries.push(BackupEntry { input_path: path, output_path: String::new(), is_file: false, hash: String::new() });
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

/// Unwraps consumed Arc<Mutex<Vec<BackupDir>>> (first Arc to Mutex, then Mutex to Vec) and returns owned Vec<BackupDir> or an error if Vec couldn't be acquired.
/// # Example (works only on Linux, test may fail if your /usr/include/bash is different):
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use std::sync::{Arc, Mutex};
/// use flash_backup::backups::traits::backup_input::arc_to_inner;
///
/// let dirs = vec![BackupDir { root_input: String::from("/usr/include/bash"), root_output: String::new(), files: 0, folders: 0, backup_entries: vec![] }];
/// let backup_dirs = Arc::new(Mutex::new(dirs));
/// let backup_dirs = arc_to_inner(backup_dirs).unwrap();
/// assert_eq!(backup_dirs[0].root_input, "/usr/include/bash");
/// ```
pub fn arc_to_inner(arc: Arc<Mutex<Vec<BackupDir>>>) -> Result<Vec<BackupDir>, String> {
    match Arc::try_unwrap(arc) {
        Ok(mutex) => {
            match mutex.into_inner() {
                Ok(dir) => {
                    Ok(dir)
                }
                Err(e) => {
                    let message = format!("Couldn't unwrap Mutex to inner: {}", e);
                    Err(message)
                }
            }
        }
        Err(_) => Err(String::from("Couldn't unwrap Arc to Mutex"))
    }
}
