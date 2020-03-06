use flash_lib::create_backup;

use clap::{App, Arg};
use std::path::Path;

fn main() {
    let matches = App::new("Flash Backup")
        .version("0.2")
        .author("Szymon Trybała <szymon.trybala@protonmail.com")
        .about("Simple app to help copying set of folders to another folder, like for example flash drive")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("CONFIG")
            .help("Loads your custom .config.json file. If not provided, program will ask you for input and output paths, max amount of held backups, and default mode"))
        .arg(Arg::with_name(".ignore")
            .short("i")
            .long(".ignore")
            .value_name("IGNORE")
            .help("Loads your custom ..ignore file. If not provided program will copy every folder and file from source directories"))
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .value_name("MODE")
            .help("Sets copy mode - you can choose between multiple (m, multiple) and cloud (c, cloud) modes. If not provided, mode will be loaded from .config.json file"))
        .get_matches();

    let custom_config_path = matches.value_of("config").unwrap_or("");
    let custom_ignore_path = matches.value_of(".ignore").unwrap_or("");
    let custom_mode= matches.value_of("mode").unwrap_or("");

    check_args(custom_config_path, custom_ignore_path, custom_mode);
    create_backup(custom_config_path, custom_ignore_path, custom_mode);
}

fn check_args(config: &str, ignore: &str, mode: &str) {
    if config.len() > 0 {
        let config_path = Path::new(config);
        if !(config_path.exists() && config_path.is_file()) {
            panic!("Wrong path to config file");
        }
    }

    if ignore.len() > 0 {
        let ignore_path = Path::new(ignore);
        if !(ignore_path.exists() && ignore_path.is_file()) {
            panic!("Wrong path to ignore file");
        }
    }

    if mode.len() > 0 {
        if !(mode == "m" || mode == "multiple" || mode == "c" || mode == "cloud") {
            panic!("Unrecognized argument for mode selection");
        }
    }
}