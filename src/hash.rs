use std::io::Read;
use std::fs::File;
use std::hash::Hasher;
use twox_hash::XxHash64;

// Hash algorithm enum
#[derive(Clone, Copy)]
pub enum HashAlgorithm {
    Md5,
    XxHash64,
}

impl HashAlgorithm {
    // Parse from command line argument
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "md5" => Some(HashAlgorithm::Md5),
            "xxhash" | "xxhash64" => Some(HashAlgorithm::XxHash64),
            _ => None,
        }
    }

    // Get algorithm name for output
    pub fn name(&self) -> &str {
        match self {
            HashAlgorithm::Md5 => "md5",
            HashAlgorithm::XxHash64 => "xxhash64",
        }
    }
}

/*
 * Compute file hash with specified algorithm
 */
pub fn compute_file_hash(file_path: &String, inner_path: &String, algorithm: HashAlgorithm) -> String {
    let mut buffer = Vec::new();
    let mut f = File::open(file_path.to_owned()).unwrap();
    f.read_to_end(&mut buffer).unwrap();

    match algorithm {
        HashAlgorithm::Md5 => {
            // file md5
            let digest = md5::compute(&buffer);
            let file_hash = format!("{:x}", digest);

            // file identifier: md5(md5(file) + file_path)
            let combined = (file_hash + &inner_path).into_bytes();
            let id_digest = md5::compute(&combined);

            format!("{:x}", id_digest)
        }
        HashAlgorithm::XxHash64 => {
            // file xxhash64
            let mut hasher = XxHash64::default();
            hasher.write(&buffer);
            let file_hash = hasher.finish();

            // file identifier: hash(hash(file) + file_path)
            let mut id_hasher = XxHash64::default();
            id_hasher.write(&file_hash.to_le_bytes());
            id_hasher.write(inner_path.as_bytes());

            format!("{:016x}", id_hasher.finish())
        }
    }
}

/*
 * Compute buffer hash with specified algorithm
 */
pub fn compute_buffer_hash(buffer: &[u8], algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::Md5 => {
            let digest = md5::compute(buffer);
            format!("{:x}", digest)
        }
        HashAlgorithm::XxHash64 => {
            let mut hasher = XxHash64::default();
            hasher.write(buffer);
            format!("{:016x}", hasher.finish())
        }
    }
}

