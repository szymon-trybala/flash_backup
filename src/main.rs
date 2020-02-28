mod user_data;
mod copying;
mod serialization;
mod multiple_mode;

use user_data::Paths;
use crate::copying::Copying;
use crate::serialization::{Serialization, BackupMetadata};
use std::collections::HashMap;
use crate::multiple_mode::Multiple;

fn main() {
    // // TODO AT FINISH CHECK ALL unwrap() and expect()
    //
    // let paths = Paths::new();
    // let mut copying = Copying::new(paths.input_path.as_str());
    //
    // match Paths::load_ignores() {
    //     Err(e) => {
    //         println!("{}", &e);
    //     }
    //
    //     Ok(ignores) => {
    //         if ignores.0.len() > 0 {
    //             if let Err(_) = copying.exclude_folder(&ignores.0) {
    //                 println!("Error excluding folders, program will copy every folder!");
    //             }
    //         }
    //         if ignores.1.len() > 0 {
    //             if let Err(_) = copying.exclude_files_with_extension(&ignores.1) {
    //                 println!("Error excluding extensions, program will copy files with every extensions!");
    //             }
    //         }
    //     }
    // }
    // copying.copy(&paths.output_path);
    // let mut serialization = Serialization::new(copying.output_files_paths).unwrap_or(Serialization { map: HashMap::new(), metadata: BackupMetadata::new()});
    // match serialization.serialize_to_json(paths.output_path) {
    //     Ok(_) => {
    //         println!("JSON file map succesfully saved in root output folder!");
    //     }
    //     Err(e) => {
    //         println!("Serialization to JSON: {}", e);
    //     }
    // }

    let mut multiple = Multiple::new();
    multiple.find_backups(String::from("/home/szymon/Downloads/2")).unwrap();
}