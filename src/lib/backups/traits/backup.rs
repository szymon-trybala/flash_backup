/// Provides function that is intended to do backup process.
///
/// Trait should be implemented in main backup structs (any mode).
///
/// Implementation of this trait should require filled basic metadata in BackupMap.
pub trait Backup {
    /// Function that should handle backup process, intended to be implemented in backup structs (any mode).
    ///
    /// Implementation of this trait should require filled basic metadata in BackupMap.
    fn backup(&mut self) -> Result<(), String>;
}