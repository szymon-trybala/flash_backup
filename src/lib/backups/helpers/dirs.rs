use std::path::MAIN_SEPARATOR;

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