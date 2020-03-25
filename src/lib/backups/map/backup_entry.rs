#[derive(Clone)]
pub struct BackupEntry {
    pub input_path: String,
    pub output_path: String,
    pub is_file: bool,
    pub hash: String,
}

impl BackupEntry {
    pub fn new() -> BackupEntry {
        BackupEntry { input_path: String::new(), output_path: String::new(), is_file: false, hash: String::new() }
    }
}