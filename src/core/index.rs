use std::path::{Path, PathBuf};

use crate::core::io::{create_file, read_file_str};

pub struct Index {
    pub hash: String
}

impl Index {
    pub fn from_hash(hash: String) -> Self {
        Index { hash }
    }

    pub fn new() -> Self {
        let hash = read_file_str(&Self::index_path().to_path_buf());
        Self::from_hash(hash)
    }

    pub fn index_path() -> PathBuf {
        Path::new("./.aequorea/index").to_path_buf()
    }

    pub fn write_index(&self) {
        create_file(&Self::index_path().to_path_buf(), self.hash.as_bytes());
    }
}
