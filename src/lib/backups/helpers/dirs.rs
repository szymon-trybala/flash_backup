use std::path::{MAIN_SEPARATOR, Path};
use crate::backups::map::backup_map::BackupMap;
use walkdir::WalkDir;
use crate::{S_MAP, S_SEPARATOR};
use std::fs::{File, remove_dir, remove_file};
use std::io::BufReader;
use crate::backups::map::backup_dir::BackupDir;
use crate::backups::map::backup_entry::BackupEntry;
use std::error::Error;

#[cfg(test)]
mod tests {
    use crate::backups::helpers::dirs::get_last_subdir;

    #[test]
    fn test_get_last_subdir() {
        let path = String::from("/usr/lib/firefox");
        let last = get_last_subdir(&path).unwrap();
        assert_eq!(last, String::from("firefox"));    }
}

/// Returns last section from path (for example "/usr/lib/firefox" returns "firefox", and "/usr/lib/a.txt" returns "a.txt".
///
/// Returns error if path isn't folder getting last subdir fails or if last subdir is empty string.
///
/// # Example:
/// To pass test you need to provide your own paths.
/// ```
/// use flash_backup::backups::helpers::dirs::get_last_subdir;
/// let path = String::from("/usr/lib/firefox");
/// let last = get_last_subdir(&path).unwrap();
/// assert_eq!(last, String::from("firefox"));
/// ```
pub fn get_last_subdir(path: &String) -> Result<String, String> {
    let path_str = path.as_str();
    let path_splitted: Vec<&str> = path_str.split(MAIN_SEPARATOR).collect();
    match path_splitted.last() {
        Some(last) => {
            if last.is_empty() {
                return Err(String::from("Last subdir is empty, skipping..."));
            }
            Ok(last.to_string())
        },
        None => {
            let message = format!("Error while converting folder paths in {}, skipping...", path);
            Err(message)
        }
    }
}

pub fn find_previous_backups(dir: &String) -> Result<Vec<BackupMap>, String> {
    let mut previous_maps = vec![];
    let path = Path::new(dir);
    if !path.is_dir() {
        return Err(String::from("Trying to find maps in path that isn't directory"));
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().ends_with(S_MAP) {
            match File::open(entry.path()) {
                Err(e) => println!("Error reading possible map, it will not count: {}", e),
                Ok(previous_map) => {
                    let buf_reader = BufReader::new(previous_map);
                    match serde_json::from_reader(buf_reader) {
                        Ok(previous_map) => previous_maps.push(previous_map),
                        Err(e) => println!("Found not valid possible map, it will not count: {}", e)
                    }
                }
            }
        }
    }
    Ok(previous_maps)
}

pub fn delete_folder_with_content(path: &String, map_folder: &BackupDir) -> Result<usize, Box<dyn Error>> {
    let mut entries: usize = 0;
    let path_with_separator = String::from(path) + S_SEPARATOR;
    let folder_content: Vec<BackupEntry> = map_folder.backup_entries.iter().filter(|x| x.input_path.starts_with(&path_with_separator)).cloned().collect();
    if folder_content.is_empty() {
        remove_dir(path)?;
        return Ok(entries);
    }

    for item in &folder_content {
        if item.is_file {
            match remove_file(&item.output_path) {
                Ok(_) => entries += 1,
                Err(e) => {
                    println!("Error while removing file {}: {} - skipping...", &item.output_path, e);
                    continue;
                }
            }
        } else {
            if let Err(e) = delete_folder_with_content(&item.output_path, &map_folder) {
                return Err(e);
            }
            remove_dir(&item.output_path)?;
            entries += 1;
        }
    }
    entries += 1;
    Ok(entries)
}