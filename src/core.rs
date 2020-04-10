pub mod index;
pub mod io;
pub mod object;

use std::path::{Path, PathBuf};

use object::{Object, ObjectStatus};

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

    pub fn exclude_entry(path: &PathBuf) -> bool {
        path.ends_with("target") | path.ends_with(".git") | path.ends_with(".aequorea")
    }
}

pub fn current_path() -> PathBuf {
    Path::new(".").canonicalize().unwrap()
}

// fn contain_child(parent: &PathBuf, child: &PathBuf) -> bool {
//     child.starts_with(parent) & !(parent.starts_with(child))
// }

// pub fn first_add(path: PathBuf) {
//     let object = Object::from_path(path);
//     let path = object.path();
//     let mut p = Path::new(&path).canonicalize().unwrap();
//     let mut child = object;
//     while contain_child(&current_path(), &p) {
//         println!("{:?}", child.path());
//         let parent_path = p.parent().unwrap();
//         let parent = Object::Tree {
//             path: parent_path.to_path_buf(),
//             children: vec![child],
//             status: ObjectStatus::Created,
//         };
//         p = parent_path.to_path_buf();
//         child = parent;
//     }
//     let object = child;
//     object.write();
// }
