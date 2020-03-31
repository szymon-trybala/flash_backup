use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum BackupMode {
    Multiple,
    Cloud,
}