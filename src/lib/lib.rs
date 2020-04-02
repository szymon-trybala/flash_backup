pub mod backups;
pub mod config;

/// Default path separator (slash in Unix, backslash in Windows).
#[cfg(unix)]
pub static S_SEPARATOR: &str = "/";
/// Default path separator (slash in Unix, backslash in Windows).
#[cfg(windows)]
pub static S_SEPARATOR: &str = "\\";
/// Default name of file containing backup map.
pub static S_MAP: &str = ".map.json";
/// Default name of file containing program's configuration file.
pub static S_CONFIG: &str = ".config.json";
/// Default name of file containing file with ignores.
pub static S_IGNORE: &str = ".ignore";


// TODO - CREATE NICE README
// TODO - WITH EVERY PANIC SHOULD BE MESSAGE "PROGRAM WILL STOP"
// TODO - check all tests, some may fail now
