use std::fs;
use std::fs::File;
use std::process::exit;

fn main() {
    // Testing copy
    let new_file_name = "output_temp_test_text.txt";
    let _new_file = File::create(new_file_name);
    match fs::copy("input_temp_test_text.txt", new_file_name) {
        Err(_) => {
            println!("Error while copying");
            exit(-1);
        },
        Ok(_) => println!("OK"),
    }
}
