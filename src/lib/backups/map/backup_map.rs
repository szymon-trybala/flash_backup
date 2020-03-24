use crate::backups::map::backup_mode::BackupMode;
use crate::backups::map::backup_dir::BackupDir;

pub struct BackupMap {
    id: String,
    timestamp: usize,
    backup_mode: BackupMode,
    files: usize,
    folders: usize,
    output_folder: String,
    input_folders: String,
    backup_dirs: Vec<BackupDir>
}

impl BackupMap {
    pub fn new() -> BackupMap {
        BackupMap { id: String::new(), timestamp: 0, backup_mode: BackupMode::Multiple, files: 0, folders: 0, output_folder: String::new(), input_folders: String::new(), backup_dirs: Vec::new() }
    }
}