# Flash backup

Simple tool written in Rust, to help quickly backup previously selected folders/files to another folder (which is usually thumb drive or external disk intended for backups).

*Features:*
* Pseudo-cloud mode, for "syncing" data (copying only new or modified files), that keeps only one, latest copy
* Multiple backups mode, where new copy is created for each backup - maximum amount of kept copies can be selected by user
* `.gitignore`-like function, where user can disable backing up certain folders or files - like `node_modules`
* CLI and GUI 