use std::path::Path;
use clap::{App, Arg};
use crate::config::config::Config;
use crate::backups::map::backup_map::BackupMap;

/// Gets program arguments, checks them and then returns initially filled ```BackupMap``` based on them.
///
/// Possible arguments are: ```-n``` / ```--new``` for confirming that user wants to create new config and overwrite existing one (with possible values 0 or 1),
/// ```-c``` / ```--config``` with path as value, to load config file from this path, or ```-i``` / ```--ignore```, which does the same thing with ignore file.
///
/// Function may panic if arguments are invalid.
pub fn args_to_map() -> BackupMap {
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
        .get_matches();

    let custom_config_path = matches.value_of("config").unwrap_or("");
    let custom_ignore_path = matches.value_of(".ignore").unwrap_or("");
    let run_new_config = matches.value_of("new").unwrap_or("0");
    let map = check_and_send_args(run_new_config, custom_config_path, custom_ignore_path);
    return map;
}

/// Checks integrity of arguments, if every one of them is ok it gets map from ```Config``` struct and returns it.
///
/// Panics if arguments are invalid - ```run_new_config``` has to be 0 or 1 (in other cases it will be changed to 0), and ```config_path``` and ```ignore_path``` must exist and be a file, if those strings are not empty.
pub fn check_and_send_args(run_new_config: &str, config_path: &str, ignore_path: &str) -> BackupMap {
    let mut run_new_config = run_new_config.trim();
    match run_new_config {
        "0" => {},
        "1" => {},
        _ => run_new_config = "0",
    }

    let mut run_new_config_usize: usize = 0;
    match run_new_config.parse::<usize>() {
        Ok(arg) => run_new_config_usize = arg,
        Err(_) => panic!("Argument 'new' isn't valid number, provide 0 or 1!. Program will stop"),
    }

    if config_path.len() > 0 {
        let config_path = Path::new(config_path);
        if !(config_path.exists() && config_path.is_file()) {
            panic!("Wrong path to config file. Program will stop");
        }
    }

    if ignore_path.len() > 0 {
        let ignore_path = Path::new(ignore_path);
        if !(ignore_path.exists() && ignore_path.is_file()) {
            panic!("Wrong path to ignore file. Program will stop");
        }
    }

    let mut config = Config::new();
    let map = config.create_backup_map(run_new_config_usize, config_path, ignore_path);
    return map;
}