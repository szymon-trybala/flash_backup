use crate::backups::map::backup_map::BackupMap;

/// Provides function that should fill BackupMap with output data.
///
/// Backup Modes differ in the way they handle creating output files, so every mode should implement it on its own way.
///
/// Using this trait should require created BackupMap with all input data, after ignoring etc.
pub trait BackupOutput {
    /// Fills BackupMap with output data.
    ///
    /// Requires owned BackupMap, fills it with data and returns it. This function should never return error.
    ///
    /// Implementations should require created BackupMap with all input data, after ignoring etc.
    fn create_output_map(map: BackupMap) -> BackupMap;
}