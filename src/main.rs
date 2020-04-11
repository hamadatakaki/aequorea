extern crate aequorea;
use aequorea::core::current_path;
use aequorea::core::index::Index;
use aequorea::core::object::{Object, ObjectDebug};
use aequorea::command::add::add;

fn main() {
    // let obj = Object::from_path(current_path());
    // obj.write();
    // let index = Index::from_hash(obj.hash());
    // // index.write_index();

    // let index = Index::new();
    // let obj = Object::from_compressed_obj(current_path(), index.hash, String::from("tree"));

    add(current_path().to_path_buf());

    // obj.debug_print_path();
}
