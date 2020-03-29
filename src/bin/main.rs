use flash_backup::backups::modes::backup_cloud::BackupCloud;
use flash_backup::backups::traits::backup_ignore::BackupIgnore;

fn main() {
    let paths = vec![String::from("/usr/lib/firefox"), String::from("/usr/lib/code")];
    let ignore_files = vec![String::from(".so"), String::from(".json")];
    let ignore_folders = vec![String::from("/browser"), String::from("/extensions")];
    let mut cloud = BackupCloud::new(&paths);


    let len_start_firefox = cloud.map.backup_dirs[0].backup_entries.len();
    let len_start_code = cloud.map.backup_dirs[1].backup_entries.len();

    cloud.map.backup_dirs = BackupCloud::ignore_files_and_folders_parrarel(cloud.map.backup_dirs, &ignore_files, &ignore_folders).unwrap();
    let len_end_firefox = cloud.map.backup_dirs[0].backup_entries.len();
    let len_end_code = cloud.map.backup_dirs[1].backup_entries.len();

    assert_eq!(len_start_firefox - len_end_firefox, 33);
    assert_eq!(len_start_code - len_end_code, 1670);
}
