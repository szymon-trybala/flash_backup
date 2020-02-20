use walkdir::WalkDir;
use std::io;
fn main() {
    println!("Write path to folder: ");
    let mut path = String::new();
    io::stdin().read_line(&mut path);

    let mut files_vec = Vec::new();
    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        files_vec.push(entry);
    }
    for x in &files_vec {
        println!("{:?} {}", x.file_type() ,x.path().display());
    }
}
