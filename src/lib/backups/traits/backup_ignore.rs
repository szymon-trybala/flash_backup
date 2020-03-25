use crate::backups::map::backup_dir::BackupDir;
use crate::backups::map::backup_entry::BackupEntry;

pub trait BackupIgnore {
    fn ignore_files(backup_folders: Vec<BackupDir>, files_to_ignore: &Vec<String>) -> Vec<BackupDir> {
        vec![]
    }

    fn ignore_folders(backup_folders: &Vec<BackupDir>, folders_to_ignore: &Vec<String>) -> Vec<BackupDir> {
        vec![]
    }
}

pub fn ignore_extensions_single_folder(folder: &mut BackupDir, extensions_to_ignore: &Vec<String>) -> Result<usize, String> {
    // Checking input
    if folder.backup_entries.is_empty() {
        let message = format!("Folder {} is empty", &folder.root_input);
        return Err(message);
    }
    if extensions_to_ignore.is_empty() {
        return Err(String::from("No file ignores provided"));
    }

    // Ignoring extensions
    let before = folder.backup_entries.len();
    for extension in extensions_to_ignore {
        folder.backup_entries.retain(|x| !(x.is_file && x.input_path.ends_with(extension)));
    }

    // Counting ignored, returning amount
    let after = folder.backup_entries.len();
    Ok(before - after)
}

pub fn ignore_folders_single_folder(folder: &mut BackupDir, folders_to_ignore: &Vec<String>) -> Result<(usize, usize), String> {
    // Checking input
    if folder.backup_entries.is_empty() {
        let message = format!("Folder {} is empty", &folder.root_input);
        return Err(message);
    }
    if folders_to_ignore.is_empty() {
        return Err(String::from("No folder ignores provided"));
    }

    // Ignoring
    let folders_start = folder.backup_entries.iter().filter(|x| !x.is_file).count();
    let files_start = folder.backup_entries.iter().filter(|x| x.is_file).count();

    for folder_to_ignore in folders_to_ignore {
        let excluded_folders: Vec<BackupEntry> = folder.backup_entries.clone().into_iter().filter(|x| !x.is_file && x.input_path.contains(folder_to_ignore)).collect();
        folder.backup_entries.retain(|x| !(!x.is_file && x.input_path.contains(folder_to_ignore))); // Ignoring folders
        for excluded_folder in excluded_folders {
            folder.backup_entries.retain(|x| !(x.input_path.starts_with(&excluded_folder.input_path)));        // Ignoring files in folders
        }
    }

    // Counting ignored, returning amount
    let folders_end = folder.backup_entries.iter().filter(|x| !x.is_file).count();
    let files_end = folder.backup_entries.iter().filter(|x| x.is_file).count();
    Ok((files_start - files_end, folders_start - folders_end))
}