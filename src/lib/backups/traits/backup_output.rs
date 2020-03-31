use crate::backups::map::backup_map::BackupMap;

pub trait BackupOutput {
    fn create_output_map(map: BackupMap) -> BackupMap;
}