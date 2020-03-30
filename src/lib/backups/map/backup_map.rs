use crate::backups::map::backup_mode::BackupMode;
use crate::backups::map::backup_dir::BackupDir;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BackupMap {
    pub id: String,
    pub timestamp: usize,
    pub backup_mode: BackupMode,
    pub files: usize,
    pub folders: usize,
    pub output_folder: String,
    pub input_folders: String,
    pub backup_dirs: Vec<BackupDir>
}

impl BackupMap {
    pub fn new() -> BackupMap {
        BackupMap { id: String::new(), timestamp: 0, backup_mode: BackupMode::Multiple, files: 0, folders: 0, output_folder: String::new(), input_folders: String::new(), backup_dirs: Vec::new() }
    }
}