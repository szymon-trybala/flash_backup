use crate::backups::map::backup_map::BackupMap;
use crate::backups::map::backup_mode::BackupMode;
use crate::backups::modes::backup_multiple::BackupMultiple;
use crate::backups::traits::backup::Backup;
use crate::backups::modes::backup_cloud::BackupCloud;

pub mod backups;
pub mod config;

/// Default path separator (slash in Unix, backslash in Windows).
#[cfg(unix)]
pub static S_SEPARATOR: &str = "/";
/// Default path separator (slash in Unix, backslash in Windows).
#[cfg(windows)]
pub static S_SEPARATOR: &str = "\\";
/// Default name of file containing backup map.
pub static S_MAP: &str = ".map.json";
/// Default name of file containing program's configuration file.
pub static S_CONFIG: &str = ".config.json";
/// Default name of file containing file with ignores.
pub static S_IGNORE: &str = ".ignore";

pub fn make_backup(map: BackupMap) {
    match map.backup_mode {
        BackupMode::Multiple => {
            let mut multiple = BackupMultiple::new(map);
            multiple.backup();
        }
        BackupMode::Cloud => {
            let mut cloud = BackupCloud::new(map);
            cloud.backup();
        }
    }
}

// TODO - CREATE NICE README
// TODO - WITH EVERY PANIC SHOULD BE MESSAGE "PROGRAM WILL STOP"
// TODO - check all tests, some may fail now
