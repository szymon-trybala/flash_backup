use crate::backups::map::backup_entry::BackupEntry;
use serde::{Deserialize, Serialize};

/// Contains data of one folder.
///
/// Intended to be used with BackupMap.
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct BackupDir {
    pub root_input: String,
    pub root_output: String,
    pub files: usize,
    pub folders: usize,
    pub backup_entries: Vec<BackupEntry>
}

impl BackupDir {
    /// Creates new instance of BackupDir, with all values empty or equal to zero.
    pub fn new() -> BackupDir {
        BackupDir { root_input: String::new(), root_output: String::new(), files: 0, folders: 0, backup_entries: Vec::new() }
    }
}