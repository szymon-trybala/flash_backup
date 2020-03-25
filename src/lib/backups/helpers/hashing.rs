use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use meowhash::MeowHasher;
use ring::digest::{Context, SHA256};
use digest::Digest;

/// Function generate hash of a file and return it as a string. Uses meow_hash, extremally fast non-cryptographic hasing method. This method of hashing is prefered in Flash Backup.
///
/// Returns error if file to which the path leads doesn't exist or is empty, exists but can't be opened, or if an error occurs during hashing.
/// # Example (only for Linux, test may not pass if your bash file is different):
/// ```
/// use flash_backup::backups::helpers::hashing::generate_hash_meow_hash;
/// let path = "/usr/bin/bash";
/// let hash = generate_hash_meow_hash(path).unwrap();
/// assert_eq!(hash, "1f0b7365561cc1809ad6016549e336234cd13758ef49fe5a474157c469f5a70533b1bc0c119e9bb0c552bcc0b80cd90c209c3b51af011fd4aa0ab474a1fb944b32f0dc02dd478794f52ad4754150669f4215152b3c1ae192b7db0b1899fc33c299d4f3b3c33a83f010d1d093297a7a50bad84806d81c87752298483f60de977b");
/// ```
pub fn generate_hash_meow_hash(path: &str) -> Result<String, String> {
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

/// Function generate hash of a file and return it as a string. Uses SHA-256, standard and widely used non-cryptographic hasing method.
///
/// Returns error if file to which the path leads doesn't exist or is empty, exists but can't be opened, or if an error occurs during hashing.
/// # Example (only for Linux, test may not pass if your bash file is different):
/// ```
/// use flash_backup::backups::helpers::hashing::generate_hash_sha256;
/// let path = "/usr/bin/bash";
/// let hash = generate_hash_sha256(path).unwrap();
/// assert_eq!(hash, "fa834f012927e241e18ac016ddc3b352f848c0bd5fe98f21a1355c48b5518211");
/// ```
pub fn generate_hash_sha256(path: &str) -> Result<String, String> {
    let as_path = Path::new(path);
    if !(as_path.exists() && as_path.is_file()) {
        let message = format!("Path {} doesn't exist or isn't a file", path);
        return Err(message);
    }

    match File::open(path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut context = Context::new(&SHA256);
            let mut buffer = [0; 1024];

            loop {
                match reader.read(&mut buffer) {
                    Ok(count) => {
                        if count == 0 {
                            break;
                        }
                        context.update(&buffer[..count])
                    }
                    Err(e) => {
                        let message = format!("Couldn't hash file {}: {}", path, e);
                        return Err(message);
                    }
                }

            }
            let digest = context.finish();


            Ok(hex::encode(digest.as_ref()))
        }
        Err(e) => {
            let message = format!("Couldn't open file {} to generate its hash: {}", path, e);
            Err(message)
        }
    }
}