use std::path::PathBuf;

use crate::core::io::{create_file, read_file_str};
use crate::core::current_path;

pub struct Index {
    hash: String
}

impl Index {
    pub fn from_hash(hash: String) -> Self {
        Index { hash }
    }

    pub fn new() -> Option<Self> {
        let hash = read_file_str(&Self::index_path().to_path_buf());
        if !hash.is_empty() {
            Some(Self::from_hash(hash))
        } else {
            None
        }
    }

    pub fn index_path() -> PathBuf {
        let mut path = current_path();
        path.push(".aequorea/index");
        path.to_path_buf()
    }

    pub fn hash(&self) -> String {
        String::from(&self.hash)
    }

    pub fn write(&self) {
        create_file(&Self::index_path().to_path_buf(), self.hash.as_bytes());
    }
}
