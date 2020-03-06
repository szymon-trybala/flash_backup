use flash_lib::create_backup;

use clap::{App, Arg};

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
            .help("Sets copy mode - you can choose between multiple (m, mul, multiple) and cloud (c, cld, cloud) modes. If not provided, mode will be loaded from .config.json file"))
        .get_matches();

    let custom_config_path = matches.value_of("config").unwrap_or("");
    println!("Config path: {}", custom_config_path);

    let custom_ignore_path = matches.value_of(".ignore").unwrap_or("");
    println!("Ignore path: {}", custom_ignore_path);

    let custom_mode= matches.value_of("mode").unwrap_or("");
    println!("Mode: {}", custom_mode);

    // TODO - check if values are existing files, pass custom paths to functions loading config and ignores, create function to run multiple_mode or cloud_mode based on input
    create_backup();
}