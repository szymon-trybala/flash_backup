use crate::backups::traits::backup_input::BackupInput;
use crate::backups::traits::backup_ignore::BackupIgnore;
use crate::backups::map::backup_map::BackupMap;
use crate::backups::traits::backup_copy::BackupCopy;
use crate::backups::map::backup_mode::BackupMode;

pub struct BackupCloud {
    pub map: BackupMap
}

impl BackupCloud {
    pub fn new(paths: &Vec<String>) -> BackupCloud {
        let folders = BackupCloud::create_input_maps(paths);
        let mut map = BackupMap::new(BackupMode::Cloud);
        map.backup_dirs = folders;
        BackupCloud { map }
    }
}

impl BackupInput for BackupCloud {}
impl BackupIgnore for BackupCloud {}
impl BackupCopy for BackupCloud {}
