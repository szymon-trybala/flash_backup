use crate::backups::map::backup_mode::BackupMode;
use crate::backups::map::backup_dir::BackupDir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct BackupMap {
    pub id: String,
    pub timestamp: usize,
    pub backup_mode: BackupMode,
    pub max_backups: usize,
    pub files: usize,
    pub folders: usize,
    pub output_folder: String,
    pub input_folders: Vec<String>,
    pub ignore_extensions: Vec<String>,
    pub ignore_folders: Vec<String>,
    pub backup_dirs: Vec<BackupDir>
}

impl BackupMap {
    pub fn new(mode: BackupMode) -> BackupMap {
        BackupMap { id: String::new(), timestamp: 0, backup_mode: mode, max_backups: 1, files: 0, folders: 0, output_folder: String::new(), input_folders: vec![], ignore_extensions: vec![], ignore_folders: vec![], backup_dirs: Vec::new() }
    }

    /// BackupMode has been already filled
    pub fn generate_metadata(&mut self) {
        self.id = Uuid::new_v4().to_string();
        self.timestamp = Utc::now().timestamp() as usize;
        self.files = self.backup_dirs.iter().map(|x| x.files).sum();
        self.folders = self.backup_dirs.iter().map(|x| x.folders).sum();
    }
}