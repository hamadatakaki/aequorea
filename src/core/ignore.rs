use std::path::PathBuf;

use crate::core::io::read_lines;
use crate::core::current_path;

pub struct Ignore {
    names: Vec<String>
}

impl Ignore {
    pub fn new() -> Self {
        let mut path = current_path();
        path.push(".aequoreaignore");
        let mut names = read_lines(&path)
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();
        names.push(String::from(".aequorea"));
        Self { names }
    }

    pub fn for_path(&self, path: &PathBuf) -> bool {
        let mut result = false;
        for name in self.names.as_slice() {
            result = result | path.ends_with(name.as_str());
        }
        result
    }
}
