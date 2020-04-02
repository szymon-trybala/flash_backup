use flash_backup::backups::modes::backup_multiple::BackupMultiple;
use flash_backup::backups::map::backup_map::BackupMap;
use flash_backup::backups::map::backup_mode::BackupMode;
use flash_backup::backups::traits::backup::Backup;
use flash_backup::backups::modes::backup_cloud::BackupCloud;

fn main() {
    let paths = vec![String::from("/usr/lib/firefox"), String::from("/home/szymon/Downloads/SIUPS")];
    let ignore_files = vec![String::from(".so"), String::from(".json")];
    let ignore_folders = vec![String::from("/browser"), String::from("/out")];
    let mut map = BackupMap::new(BackupMode::Cloud);
    map.input_folders = paths;
    map.ignore_extensions = ignore_files;
    map.ignore_folders = ignore_folders;
    map.max_backups = 3;
    map.output_folder = String::from("/home/szymon/Downloads/HOPS");
    let mut dupa = BackupCloud::new(map);
    dupa.backup().unwrap();
}
