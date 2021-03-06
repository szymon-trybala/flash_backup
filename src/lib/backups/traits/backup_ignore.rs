use crate::backups::map::backup_dir::BackupDir;
use crate::backups::map::backup_entry::BackupEntry;
use std::sync::{Arc, Mutex};
use scoped_threadpool::Pool;
use std::borrow::BorrowMut;
use crate::backups::helpers::multithreading::arc_to_inner;

#[cfg(test)]
mod tests {
    use crate::backups::traits::backup_input::BackupInput;
    use crate::backups::traits::backup_ignore::{ignore_extensions_single_folder, ignore_folders_single_folder, BackupIgnore};
    use crate::backups::modes::backup_cloud::BackupCloud;
    use crate::backups::map::backup_dir::BackupDir;
    use crate::backups::map::backup_entry::BackupEntry;
    use crate::backups::map::backup_map::BackupMap;
    use crate::backups::map::backup_mode::BackupMode;

    #[test]
    fn test_ignore_extensions_single_folder() {
        let mut dir = BackupDir {files: 1, folders: 1, root_input: String::new(), root_output: String::new(), backup_entries: vec![
            BackupEntry {input_path: String::from("/home/user/Downloads/X/node_modules"), output_path: String::from("/home/user/Downloads/backup/node_modules"), is_file: false, hash: String::new() },
        BackupEntry{ input_path: String::from("/home/user/Downloads/X/node_modules/123.js"), output_path: String::from("/home/user/Downloads/backup/node_modules/123.json"), is_file: true, hash: String::from("12345") }]};
        ignore_folders_single_folder(&mut dir, &vec![String::from("/dupa_modules")]);
        assert_eq!(dir.backup_entries.len(), 2);
        ignore_folders_single_folder(&mut dir, &vec![String::from("/node_modules")]);
        assert_eq!(dir.backup_entries.len(), 0);
    }

    #[test]
    fn test_ignore_folders_single_folder() {
        let mut dir = BackupDir {files: 1, folders: 1, root_input: String::new(), root_output: String::new(), backup_entries: vec![
            BackupEntry {input_path: String::from("/home/user/Downloads/X/node_modules"), output_path: String::from("/home/user/Downloads/backup/node_modules"), is_file: false, hash: String::new() },
            BackupEntry{ input_path: String::from("/home/user/Downloads/X/node_modules/123.js"), output_path: String::from("/home/user/Downloads/backup/node_modules/123.json"), is_file: true, hash: String::from("12345") }]};
        ignore_extensions_single_folder(&mut dir, &vec![String::from(".ts")]);
        assert_eq!(dir.backup_entries.len(), 2);
        ignore_extensions_single_folder(&mut dir, &vec![String::from(".js")]);
        assert_eq!(dir.backup_entries.len(), 1);
    }

    #[test]
    fn test_ignore_files_and_folders_parrarel() {
        let mut dir = BackupDir {files: 1, folders: 1, root_input: String::new(), root_output: String::new(), backup_entries: vec![
            BackupEntry {input_path: String::from("/home/user/Downloads/X/node_modules"), output_path: String::from("/home/user/Downloads/backup/node_modules"), is_file: false, hash: String::new() },
            BackupEntry{ input_path: String::from("/home/user/Downloads/X/node_modules/123.js"), output_path: String::from("/home/user/Downloads/backup/node_modules/123.json"), is_file: true, hash: String::from("12345") }]};

        let res = BackupCloud::ignore_files_and_folders_parrarel(vec![dir.clone()], &vec![String::from(".ts")], &vec![String::from("/dupa_modules")]);
        assert_eq!(res[0].backup_entries.len(), 2);
        let res = BackupCloud::ignore_files_and_folders_parrarel(vec![dir], &vec![String::from(".js")], &vec![String::from("/node_modules")]);
        assert_eq!(res[0].backup_entries.len(), 0);
    }
}

/// Provides function to ignore selected folders and files with selected extensions from **already created** BackupDirs
pub trait BackupIgnore {
    /// Deletes provided folders (with their content) and files with provided extensions from provided Vec<BackupDir>.
    ///
    /// Requires owned Vec<BackupDir>, then returns it after processing.
    ///
    /// Processing folders is done parallelly, with 1 thread for 1 folder rule, and max amount of current folders is equal to your processor thread count.
    ///
    /// Function can panic if fatal error occurs during multithreading operations and conversions - it's too dangerous to continue runtime at this point. Function returns error if provided map is empty.
    /// Minor errors while ignoring are printed and do not stop execution of program.
    ///
    /// Syntax for ignores: extensions has to start with '.', like ".json", and folders has to start with '/', regardless of the operating system. Example for folders: "/node_modules"
    ///
    /// This function requires Vec<BackupDir> with already filled input_path in every BackupEntry, and in this program it's best to use it directly after creating maps with crate::backups::traits::backup_input::BackupInput::create_input_maps.
    ///
    /// # Example:
    /// This test requires usage of struct that implements BackupInput trait, like BackupCloud or BackupMultiple.
    /// To pass test you need to provide your own paths and ignores variables, and count difference manually.
    /// ```
    ///use flash_backup::backups::map::backup_entry::BackupEntry;
    /// use flash_backup::backups::map::backup_dir::BackupDir;
    /// use flash_backup::backups::modes::backup_cloud::BackupCloud;
    /// use flash_backup::backups::traits::backup_ignore::BackupIgnore;
    /// let mut dir = BackupDir {files: 1, folders: 1, root_input: String::new(), root_output: String::new(), backup_entries: vec![
    ///BackupEntry {input_path: String::from("/home/user/Downloads/X/node_modules"), output_path: String::from("/home/user/Downloads/backup/node_modules"), is_file: false, hash: String::new() },
    ///BackupEntry{ input_path: String::from("/home/user/Downloads/X/node_modules/123.js"), output_path: String::from("/home/user/Downloads/backup/node_modules/123.json"), is_file: true, hash: String::from("12345") }]};
    ///let res = BackupCloud::ignore_files_and_folders_parrarel(vec![dir.clone()], &vec![String::from(".ts")], &vec![String::from("/dupa_modules")]);
    ///assert_eq!(res[0].backup_entries.len(), 2);
    ///let res = BackupCloud::ignore_files_and_folders_parrarel(vec![dir], &vec![String::from(".js")], &vec![String::from("/node_modules")]);
    ///assert_eq!(res[0].backup_entries.len(), 0);
    /// ```
    fn ignore_files_and_folders_parrarel(backup_dirs: Vec<BackupDir>, extensions_to_ignore: &Vec<String>, folders_to_ignore: &Vec<String>) -> Vec<BackupDir> {
        let mut backup_dirs = Arc::new(Mutex::new(backup_dirs));
        backup_dirs = ignore_extensions_parallel(backup_dirs, extensions_to_ignore);
        backup_dirs = ignore_folders_parallel(backup_dirs, folders_to_ignore);

        match arc_to_inner(backup_dirs) {
            Ok(dirs) => {
                if dirs.is_empty() {
                    println!("Error while trying to create input maps - all maps are empty");
                }
                dirs
            }
            Err(e) => {
                let message = format!("Fatal error while trying to create input maps - {}. Program will stop", e);
                panic!(message);
            }
        }
    }
}

/// Ignores files with provided extensions from provided BackupDir.
///
/// Doesn't consume the BackupDir. Returns Ok if no errors occur, else returns String with error info.
///
/// Used by flash_backup::traits::backup_ignore::BackupIgnore::ignore_files_and_folders_parrarel
///
/// Extensions syntax: starts with '.', for example ".json"
///
/// # Example:
/// This test requires usage of struct that implements BackupInput trait, like BackupCloud or BackupMultiple.
/// To pass test you need to provide your own paths and ignores variables, and count difference manually.
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use flash_backup::backups::map::backup_entry::BackupEntry;
/// use flash_backup::backups::traits::backup_ignore::ignore_folders_single_folder;
/// let mut dir = BackupDir {files: 1, folders: 1, root_input: String::new(), root_output: String::new(), backup_entries: vec![
/// BackupEntry {input_path: String::from("/home/user/Downloads/X/node_modules"), output_path: String::from("/home/user/Downloads/backup/node_modules"), is_file: false, hash: String::new() },
/// BackupEntry{ input_path: String::from("/home/user/Downloads/X/node_modules/123.js"), output_path: String::from("/home/user/Downloads/backup/node_modules/123.json"), is_file: true, hash: String::from("12345") }]};
/// ignore_folders_single_folder(&mut dir, &vec![String::from("/dupa_modules")]);
/// assert_eq!(dir.backup_entries.len(), 2);
/// ignore_folders_single_folder(&mut dir, &vec![String::from("/node_modules")]);
/// assert_eq!(dir.backup_entries.len(), 0);
/// ```
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

/// Ignores folders with provided names (and their content) from provided BackupDir.
///
/// Doesn't consume the BackupDir. Returns Ok if no errors occur, else returns String with error info.
///
/// Used by flash_backup::traits::backup_ignore::BackupIgnore::ignore_files_and_folders_parrarel
///
/// Folders syntax: always starts with '/', regardless of the operating system. Example: "/node_modules"
///
/// # Example:
/// This test requires usage of struct that implements BackupInput trait, like BackupCloud or BackupMultiple.
/// To pass test you need to provide your own paths and ignores variables, and count difference manually.
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use flash_backup::backups::map::backup_entry::BackupEntry;
/// use flash_backup::backups::traits::backup_ignore::ignore_extensions_single_folder;
/// let mut dir = BackupDir {files: 1, folders: 1, root_input: String::new(), root_output: String::new(), backup_entries: vec![
/// BackupEntry {input_path: String::from("/home/user/Downloads/X/node_modules"), output_path: String::from("/home/user/Downloads/backup/node_modules"), is_file: false, hash: String::new() },
/// BackupEntry{ input_path: String::from("/home/user/Downloads/X/node_modules/123.js"), output_path: String::from("/home/user/Downloads/backup/node_modules/123.json"), is_file: true, hash: String::from("12345") }]};
/// ignore_extensions_single_folder(&mut dir, &vec![String::from(".ts")]);
/// assert_eq!(dir.backup_entries.len(), 2);
/// ignore_extensions_single_folder(&mut dir, &vec![String::from(".js")]);
/// assert_eq!(dir.backup_entries.len(), 1);
/// ```
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

/// Ignores files with provided extensions from provided Arc<Mutex<Vec<BackupDir>>>.
///
/// It runs parallelly - 1 thread per 1 folder, with max amount of working threads equal to your computer thread count. For each thread it executes the ignore_extensions_single_folder() function.
///
/// Requires owned Arc<Mutex<Vec<BackupDir>>>, then returns it after processing.
///
/// May panic while locking Arc, if error occurs in one thread it is printed and other threads continue to work, folder structure is preserved.
///
/// Used by flash_backup::traits::backup_ignore::BackupIgnore::ignore_files_and_folders_parrarel
///
/// Extensions syntax: has to start with ".", for example: ".json"
pub fn ignore_extensions_parallel(dirs: Arc<Mutex<Vec<BackupDir>>>, extensions_to_ignore: &Vec<String>) -> Arc<Mutex<Vec<BackupDir>>> {
    // Checking input
    let dirs_ref = Arc::clone(&dirs);
    if dirs_ref.lock().unwrap().is_empty() {
        println!("No dirs to ignore from provided!");
        return dirs;
    }
    if extensions_to_ignore.is_empty() {
        println!("No extension to ignore found");
        return dirs;
    }

    // Creating necessary variables
    println!("Ignoring extensions...");
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
                    println!("Error while ignoring files in {}: {}", &dirs_temp[i].root_input, e);
                }
            });
        }
    });
    println!("Ignored extensions");
    dirs
}

/// Ignores folders with provided names (and their content) from provided Arc<Mutex<Vec<BackupDir>>>.
///
/// It runs parallelly - 1 thread per 1 folder, with max amount of working threads equal to your computer thread count. For each thread it executes the ignore_folders_single_folder() function.
///
/// Requires owned Arc<Mutex<Vec<BackupDir>>>, then returns it after processing.
///
/// May panic while locking Arc, if error occurs in one thread it is printed and other threads continue to work, folder structure is preserved.
///
/// Used by flash_backup::traits::backup_ignore::BackupIgnore::ignore_files_and_folders_parrarel
/// Folders syntax: always starts with '/', regardless of the operating system. Example: "/node_modules"
pub fn ignore_folders_parallel(dirs: Arc<Mutex<Vec<BackupDir>>>, folders_to_ignore: &Vec<String>) -> Arc<Mutex<Vec<BackupDir>>> {
    // Checking input
    let dirs_ref = Arc::clone(&dirs);
    if dirs_ref.lock().unwrap().is_empty() {
        println!("No dirs to ignore from provided!");
        return dirs;
    }
    if folders_to_ignore.is_empty() {
        println!("No folders to ignore found");
        return dirs;
    }

    // Creating necessary variables
    println!("Ignoring folders...");
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
    println!("Ignored folders");
    dirs
}