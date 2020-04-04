# Flash backup

Simple tool written in Rust, to help quickly backup previously selected folders/files to another folder (which is usually thumb drive or external disk intended for backups). User needs to create config once, then creating fresh backup is the matter of one click.

Copying can be done in two ways - in pseudo-cloud mode, that "syncs" your data, copying only new or modified files, to keep only one, latest copy, or in multiple mode, where running program creates completely new copy, alongside others, in separate folder. Program controls current number of copies, and deletes redundant ones, with maximum number of present copies provided by user.

Selected folders and files with selected extensions can be skipped, thanks to gitignore-like function, that excludes from folder maps entries with names matching arguments provided in `.ignore` file, which on default shoud be in the same folder as program executable.

Syntax of `.ignore` - to exclude folder, write line with slash and then its name, for example `/node_modules`. Slash should be used on every operating system. To exclude extensions, write it in line with dot on start, for example `.exe`. One line should contain only one ignore.

### Features:
* Tracking and updating files is possible thanks to folder maps saved in `.map.json` file, which stores info about every entry in backup.
* Program also checks file integrity using incredibly fast, non-cryptographic hash function called [meowhash](https://mollyrocket.com/meowhash).
* Flash backup uses multiple threads to maximize performance if you want to copy many folders. Usually amount of threads in [pool](https://crates.io/crates/scoped_threadpool) is equal to your processor's thread count, and for I/O operations it's limited to 2 or 4 at once, to not overload hard drives. 
* CLI reads arguments thanks to [clap](https://clap.rs/), and helps you create configuration with user-friendly wizard, asking for input folders, output folder, mode and maximum number of present copies. Config is also saved to `.config.json` file, so you have to provide data only once. 
* Works on Windows, Linux and macOS (use backslashes for paths in case if its Windows).

### Building and executing:
Requires installed Rust and Cargo. Flash Backup was built using version `1.42` of Rust.
```
git clone https://github.com/szymon-trybala/flash_backup.git
cd flash_backup
cargo fetch
cargo build --release
cd target/release
./flash_backup
```

### Usage and arguments
Preferred way to use Flash Backup is to copy its executable with `.ignore` and `.config.json` to external drive intended for backups, and just run it every time you feel like you want to make a backup.

**Possible arguments in CLI:**
*  `-n` or `--new` - ignores existing configuration file and creates new, overwriting previous one. To turn on this freature, provide value `1`. Example:
```bash
./flash_backup -n 1
```
* `-c` or `--config` - loads config file not from current folder, but from provided path. Example:
```bash
./flash_backup -c /home/user/Downloads/copy_of_configs/other_config.json
```
* `-i` or `--ignore` - loads ignore file not from current folder, but from provided path. Example:
```bash
./flash_backup -i /home/user/Downloads/other_ignore.txt
```