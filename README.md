# Flash backup

Simple tool written in Rust, to help quickly backup previously selected folders/files to another folder (which is usually thumb drive or external disk intended for backups).

### Features:
* Pseudo-cloud mode, for "syncing" data (copying only new or modified files), that keeps only one, latest copy
* Multiple backups mode, where executing program creates completely new copy - maximum amount of kept copies can be selected by user
* `.gitignore`-like function, where user can disable copying certain folders or files - like `node_modules`
* CLI (use subcommand --help for more)

### Installation:
```
git clone https://github.com/szymon-trybala/flash_backup.git
cd flash_backup
cargo fetch
cargo build --release
cd target/release
./flash_backup
```


