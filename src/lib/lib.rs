pub mod backups;
pub mod config;

#[cfg(unix)]
pub static S_SEPARATOR: &str = "/";
#[cfg(windows)]
pub static S_SEPARATOR: &str = "\\";
pub static S_MAP: &str = ".map.json";
pub static S_CONFIG: &str = ".config.json";
pub static S_IGNORE: &str = ".ignore";


// TODO - REMOVE ALL PUB AFTER CREATING DOCUMENTATION
// TODO - CREATE NICE README
// TODO - WITH EVERY PANIC SHOULD BE MESSAGE "PROGRAM WILL STOP"

