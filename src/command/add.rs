use std::path::PathBuf;
use std::collections::HashMap;

use crate::core::current_path;
use crate::core::index::Index;
use crate::core::object::{Object, ObjectType};

use crate::exit_process_with_error;

fn contain_child(parent: &PathBuf, child: &PathBuf) -> bool {
    child.starts_with(parent) & !(parent.starts_with(child))
}

pub fn add(path: PathBuf) {
    let path = path.canonicalize().unwrap_or_else(|e| exit_process_with_error!(1, "Inputed path does not exist: {}", e));
    let obj = Object::from_path(path);
    let mut path = obj.path();
    let mut child = obj;
    while contain_child(&current_path(), &path) {
        let parent_path = path.parent().unwrap();
        let mut children = HashMap::new();
        children.insert(child.path(), child);
        let parent = Object::Tree {
            path: parent_path.to_path_buf(),
            children
        };
        path = parent_path.to_path_buf();
        child = parent;
    }
    let new_obj = child;

    if let Some(index) = Index::new() {
        let old_obj = Object::from_compressed_obj(current_path(), index.hash(), ObjectType::Tree);
        old_obj.write();
        index.write();
    } else {
        new_obj.write();
        let index = Index::from_hash(new_obj.hash());
        index.write();
    }
}
