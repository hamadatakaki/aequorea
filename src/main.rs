extern crate aequorea;
use aequorea::core::current_path;
use aequorea::core::index::Index;
use aequorea::core::object::{Object, ObjectDebug};
use aequorea::command::add::add;

use std::path::Path;

fn main() {
    // let index = Index::new();
    // let obj = Object::from_compressed_obj(current_path(), index.hash, String::from("tree"));
    // println!("{:?}", obj.contain_hash_list());

    add(Path::new(".").to_path_buf());

    // obj.debug_print_path();
}
