extern crate aequorea;
use aequorea::core::current_path;
use aequorea::core::index::Index;
use aequorea::core::object::{Object, ObjectDebug};

fn main() {
    let obj = Object::from_path(current_path());
    obj.write();
    let index = Index::from_hash(obj.hash());
    index.write_index();
    // let index = Index::new();
    // let obj = Object::from_compressed_obj(current_path(), index.hash, String::from("tree"));
    obj.debug_print_path();
}
