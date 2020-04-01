use serde::{Deserialize, Serialize};

/// Enum to store all possible backup modes.
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum BackupMode {
    Multiple,
    Cloud,
}