use flash_backup::config::modes::cli::args_to_map;
use flash_backup::make_backup;

/// Gets initial version of map, then executes backup process.
fn main() {
    let map = args_to_map();
    make_backup(map);
}