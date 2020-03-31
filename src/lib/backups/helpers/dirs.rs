use std::path::MAIN_SEPARATOR;

pub fn get_last_subdir(path: &String) -> Result<String, String> {
    let path_str = path.as_str();
    let path_splitted: Vec<&str> = path_str.split(MAIN_SEPARATOR).collect();
    match path_splitted.last() {
        Some(last) => Ok(last.to_string()),
        None => {
            let message = format!("Error while converting folder paths in {}, skipping...", path);
            Err(message)
        }
    }
}