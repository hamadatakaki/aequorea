use std::path::Path;

extern crate aequorea;
use aequorea::core::{Object, ObjectDebug};
use aequorea::core::io;

fn main() {
    let obj = Object::from_path(Path::new(".").to_path_buf());
    obj.write();
    let path = Path::new("./.aequorea/index");
    io::create_file(&path.to_path_buf(), obj.hash().as_bytes());
    // let path = Path::new(".").canonicalize().unwrap();
    // let index_path = Path::new("./.aequorea/index");
    // let hash = io::read_file_str(&index_path.to_path_buf());
    // let obj = Object::from_compressed_obj(path, hash, String::from("tree"));
    obj.debug_print_path();
}
