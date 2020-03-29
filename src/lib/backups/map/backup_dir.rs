use crate::backups::map::backup_entry::BackupEntry;

#[derive(Clone)]
pub struct BackupDir {
    pub root_input: String,
    pub root_output: String,
    pub files: usize,
    pub folders: usize,
    pub backup_entries: Vec<BackupEntry>
}

impl BackupDir {
    pub fn new() -> BackupDir {
        BackupDir { root_input: String::new(), root_output: String::new(), files: 0, folders: 0, backup_entries: Vec::new() }
    }
}