use serde::{Deserialize, Serialize};
use crate::modes::Mode;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    pub timestamp: i64,
    pub elements: usize,
    pub mode: Mode,
    pub output_folder: String,
    pub input_folders: Vec<String>,
}

impl Metadata {
    pub fn new() -> Metadata {
        let backup_metadata = Metadata { id: String::new(), timestamp: 0, elements: 0, mode: Mode::Multiple, output_folder: String::new(), input_folders: Vec::new()};
        backup_metadata
    }
}