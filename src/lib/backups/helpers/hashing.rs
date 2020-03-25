use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use meowhash::MeowHasher;
use digest::Digest;

/// Function generate hash of a file and return it as a string. Uses meow_hash, extremally fast non-cryptographic hasing method. This method of hashing is prefered in Flash Backup.
/// Returns error if file to which the path leads doesn't exist or is empty, exists but can't be opened, or if an error occurs during hashing.
// TODO - add test
pub fn generate_hash_meow_hash(path: &String) -> Result<String, String> {
    let as_path = Path::new(path);
    if !(as_path.exists() && as_path.is_file()) {
        let message = format!("Path {} doesn't exist or isn't a file", path);
        return Err(message);
    }

    match File::open(path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut meow = MeowHasher::new();
            let mut buffer = [0; 1024];

            loop {
                match reader.read(&mut buffer) {
                    Ok(u) => {
                        if u == 0 {
                            break;
                        }
                        meow.input(&buffer[..u]);
                    }
                    Err(e) => {
                        let message = format!("Couldn't hash file {}: {}", path, e);
                        return Err(message);
                    }
                }
            }
            let result = meow.result();
            Ok(hex::encode(result.as_ref()))
        }
        Err(e) => {
            let message = format!("Couldn't open file {} to generate its hash: {}", path, e);
            Err(message)
        }
    }
}

// /// Function generate hash of a file and return it as a string. Uses SHA-256, standard and widely used non-cryptographic hasing method.
// /// Returns error if file to which the path leads doesn't exist or is empty, exists but can't be opened, or if an error occurs during hashing.
// /// // TODO - add test
// pub fn generate_hash_sha256(path: &String) -> Result<String, &'static str> {
//     let as_path = Path::new(path);
//     if !(as_path.exists() && as_path.is_file()) {
//         let message = format!("Path {} doesn't exist or isn't a file", path);
//         return Err(&message);
//     }
//
//     match File::open(path) {
//         Ok(file) => {
//             let mut reader = BufReader::new(file);
//             let mut context = Context::new(&SHA256);
//             let mut buffer = [0; 1024];
//
//             loop {
//                 match reader.read(&mut buffer) {
//                     Ok(count) => {
//                         if count == 0 {
//                             break;
//                         }
//                         context.update(&buffer[..count])
//                     }
//                     Err(e) => {
//                         let message = format!("Couldn't hash file {}: {}", path, e);
//                         return Err(&message);
//                     }
//                 }
//
//             }
//             let digest = context.finish();
//
//
//             Ok(hex::encode(digest.as_ref()))
//         }
//         Err(e) => {
//             let message = format!("Couldn't open file {} to generate its hash: {}", path, e);
//             Err(&message)
//         }
//     }
// }