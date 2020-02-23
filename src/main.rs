mod user_data;
mod copying;

use user_data::Paths;
use crate::copying::Copying;

fn main() {
    let paths = Paths::new();
    let mut copying = Copying::new(paths.input_path.as_str());

    let mut exclude_folders = Vec::new();
    exclude_folders.push("/home/szymon/Downloads/HOPS/Sample");

    let mut exclude_extensions = Vec::new();
    exclude_extensions.push(".srt");

    if let Err(_) = copying.exclude_folder(&exclude_folders) {
        println!("Error excluding folders, program will copy every folder!");
    }
    if let Err(_) = copying.exclude_files_with_extension(&exclude_extensions) {
        println!("Error excluding extensions, program will copy files with every extensions!");
    }
    copying.copy(&paths.output_path)
}