use std::path::{Path, PathBuf};

use crate::core::current_path;
use crate::core::index::Index;
use crate::core::object::{Object, ObjectStatus, ObjectDebug};

fn contain_child(parent: &PathBuf, child: &PathBuf) -> bool {
    child.starts_with(parent) & !(parent.starts_with(child))
}

pub fn add(path: PathBuf) {
    let obj = Object::from_path(path);
    let mut path = obj.path();
    let mut child = obj;
    while contain_child(&current_path(), &path) {
        println!("{:?}", child.path());
        let parent_path = path.parent().unwrap();
        let parent = Object::Tree {
            path: parent_path.to_path_buf(),
            children: vec![child],
            status: ObjectStatus::Created,
        };
        path = parent_path.to_path_buf();
        child = parent;
    }
    let new_obj = child;

    let index = Index::new();
    let old_obj = Object::from_compressed_obj(current_path(), index.hash, String::from("tree"));

    println!("old");
    old_obj.debug_print_path();
    println!("new");
    new_obj.debug_print_path();

    // new_obj.write();
}
