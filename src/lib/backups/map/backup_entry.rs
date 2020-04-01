use serde::{Deserialize, Serialize};

/// Contains data of one entry (file or folder).
///
/// Intended to be used with BackupDir.
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct BackupEntry {
    pub input_path: String,
    pub output_path: String,
    pub is_file: bool,
    pub hash: String,
}

impl BackupEntry {
    /// Creates new instance of BackupDir, with all values empty or equal to false.
    pub fn new() -> BackupEntry {
        BackupEntry { input_path: String::new(), output_path: String::new(), is_file: false, hash: String::new() }
    }
}