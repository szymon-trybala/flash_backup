use crate::backups::map::backup_dir::BackupDir;
use std::path::Path;
use walkdir::WalkDir;
use crate::backups::map::backup_entry::BackupEntry;
use crate::backups::helpers::hashing::generate_hash_meow_hash;
use std::thread;

pub trait BackupInput {
    fn create_input_maps(paths: &Vec<String>) {
        let mut backup_dirs = check_input_folders(paths);
        backup_dirs = fill_backup_dirs(backup_dirs);
    }
}


/// Returns Vec<Backup_Dir> filled with Backup_Dir for every passed valid and non-empty path, only present field in Backup_Dir's is root_input.
/// Panics if there are no valid and non-empty input folders - there is no point of continuing the program if this happens
/// # Examples:
/// Replace "szymon" with your username. Test only for Linux.
/// ```
/// let paths = vec![String::from("/home/szymon/Downloads")];
/// let result = check_input_folders(&paths);
/// assert_eq!(result[0].root_input, "home/szymon/Downloads");
fn check_input_folders(folders: &Vec<String>) -> Vec<BackupDir> {
    let mut dirs = Vec::new();

    for folder in folders {
        match Path::new(folder).read_dir().unwrap().next().is_none() {
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

fn fill_backup_dirs(dirs: Vec<BackupDir>) -> Vec<BackupDir> {
    for mut dir in dirs {
        thread::spawn(move || {
            dir = fill_single_backup_dir(dir);
        });
    }
    dirs
}


/// Fills data of one BackupDir. FUNCTION ASSUMS THAT root_input IS ALREADY FILLED, if it isn't, it returns unmodified dir.
/// Filled fields: folders, files, backup_entries (function creates Vec of BackupEntry, each one has input_path, is_file and hash filled).
///
/// Function may panic while converting DirEntry.path() to str, but it's almost impossible.
/// Function skip files for which hash couldn't be generated, user gets info about every skipped file.
// TODO - add test
fn fill_single_backup_dir(mut dir: BackupDir) -> BackupDir {
    // Checking input
    let as_path = Path::new(&dir.root_input);
    if !(!dir.root_input.is_empty() && as_path.exists() && as_path.is_file()) {
        println!("Path {} doesn't exist or isn't a file", &dir.root_input);
        return dir;
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
    }
    dir
}
