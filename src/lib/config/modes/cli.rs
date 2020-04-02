use std::path::Path;
use clap::{App, Arg};

pub fn get_args_from_cli() {
    let matches = App::new("Flash Backup")
        .version("0.9")
        .author("Szymon Trybała <szymon.trybala@protonmail.com")
        .about("Simple app to help copying set of folders to another folder, like for example flash drive")
        .arg(Arg::with_name("new")
            .short("n")
            .long("new")
            .value_name("NEW")
            .help("Creates new config, even if one is found in program's directory. To turn on this feature, add argument '-n 1'"))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("CONFIG")
            .help("Loads your custom .config.json file. If not provided, program will ask you for input and output paths, max amount of held backups, and default mode"))
        .arg(Arg::with_name(".ignore")
            .short("i")
            .long(".ignore")
            .value_name("IGNORE")
            .help("Loads your custom .ignore file. If not provided program will copy every folder and file from source directories"))
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .value_name("MODE")
            .help("Sets copy mode - you can choose between multiple (m, multiple) and cloud (c, cloud) modes. If not provided, mode will be loaded from .config.json file"))
        .get_matches();

    let custom_config_path = matches.value_of("config").unwrap_or("");
    let custom_ignore_path = matches.value_of(".ignore").unwrap_or("");
    let custom_mode = matches.value_of("mode").unwrap_or("");
    let run_new_config = matches.value_of("new").unwrap_or(0);

    check_args(run_new_config, custom_config_path, custom_ignore_path, custom_mode);
    create_backup_map(run_new_config, custom_config_path, custom_ignore_path, custom_mode);
}

pub fn check_args(run_new_config: usize, config_path: &str, ignore_path: &str, mode: &str) {
    if run_new_config != 0 || run_new_config != 1 {
        panic!("Unrecognized argument for deciding if you want to create new config");
    }

    if config_path.len() > 0 {
        let config_path = Path::new(config_path);
        if !(config_path.exists() && config_path.is_file()) {
            panic!("Wrong path to config file");
        }
    }

    if ignore_path.len() > 0 {
        let ignore_path = Path::new(ignore_path);
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