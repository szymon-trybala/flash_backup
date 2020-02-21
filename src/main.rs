mod user_data;
mod copying;

use user_data::Paths;
use crate::copying::Copying;

fn main() {
    let paths = Paths::new();
    let mut copying = Copying::new();
    copying.copy(&paths.input_path, &paths.output_path)
}