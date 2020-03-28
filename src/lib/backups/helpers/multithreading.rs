use std::sync::{Arc, Mutex};
use crate::backups::map::backup_dir::BackupDir;

/// Unwraps consumed Arc<Mutex<Vec<BackupDir>>> (first Arc to Mutex, then Mutex to Vec) and returns owned Vec<BackupDir> or an error if Vec couldn't be acquired.
/// # Example (works only on Linux, test may fail if your /usr/include/bash is different):
/// ```
/// use flash_backup::backups::map::backup_dir::BackupDir;
/// use std::sync::{Arc, Mutex};
/// use flash_backup::backups::helpers::multithreading::arc_to_inner;
///
/// let dirs = vec![BackupDir { root_input: String::from("/usr/include/bash"), root_output: String::new(), files: 0, folders: 0, backup_entries: vec![] }];
/// let backup_dirs = Arc::new(Mutex::new(dirs));
/// let backup_dirs = arc_to_inner(backup_dirs).unwrap();
/// assert_eq!(backup_dirs[0].root_input, "/usr/include/bash");
/// ```
pub fn arc_to_inner(arc: Arc<Mutex<Vec<BackupDir>>>) -> Result<Vec<BackupDir>, String> {
    match Arc::try_unwrap(arc) {
        Ok(mutex) => {
            match mutex.into_inner() {
                Ok(dir) => {
                    Ok(dir)
                }
                Err(e) => {
                    let message = format!("Couldn't unwrap Mutex to inner: {}", e);
                    Err(message)
                }
            }
        }
        Err(_) => Err(String::from("Couldn't unwrap Arc to Mutex"))
    }
}