pub mod ignore;
pub mod index;
pub mod io;
pub mod object;

use std::path::{Path, PathBuf};

pub enum Entry {
    File,
    Dir,
}

impl Entry {
    pub fn modelize_entry(path: &PathBuf) -> Option<Entry> {
        if path.is_dir() {
            Some(Entry::Dir)
        } else if path.is_file() {
            Some(Entry::File)
        } else {
            None
        }
    }
}

pub fn current_path() -> PathBuf {
    Path::new(".").canonicalize().unwrap()
}
