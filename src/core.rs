pub mod io;

use std::fs;
use std::path::{Path, PathBuf};

use super::core::io::*;

#[derive(Debug)]
pub enum ObjectStatus {
    Created,
    Existed,
    Deleted,
}

enum Entry {
    File,
    Dir,
}

// pub struct Commit {
//     tree: String,
//     parent: String,
//     message: String,
//     status: ObjectStatus,
// }

#[derive(Debug)]
pub enum Object {
    Blob {
        data: Vec<u8>,
        path: PathBuf,
        status: ObjectStatus,
    },
    Tree {
        path: PathBuf,
        children: Vec<Object>,
        status: ObjectStatus,
    }
}

pub trait ObjectDebug {
    fn debug_print_path(&self);
}

impl Object {
    pub fn from_path(path: PathBuf) -> Self {
        let path = if path.is_relative() {
            path.canonicalize().unwrap()
        } else {
            path
        };
        match modelize_entry(&path) {
            Some(Entry::File) => {
                let file = read_file_bytes(&path);
                Object::Blob {
                    data: file,
                    path,
                    status: ObjectStatus::Created,
                }
            },
            Some(Entry::Dir) => {
                let mut v = Vec::new();
                for entry in fs::read_dir(&path).unwrap() {
                    let entry_path = entry.unwrap().path();
                    if skip_entry(&entry_path) {
                        continue;
                    }
                    let object: Object = Object::from_path(entry_path);
                    v.push(object);
                }
                Object::Tree {
                    path,
                    children: v,
                    status: ObjectStatus::Created,
                }
            },
            _ => unreachable!()
        }
    }

    pub fn from_compressed_obj(path: PathBuf, hash: String, object_type: String) -> Self {
        let object_path = format!("./.aequorea/objects/{}", hash);
        let object_path = Path::new(&object_path);
        let source = read_file_bytes(&object_path.to_path_buf());
        let decompressed = decompress_by_zlib(source.as_slice());
        match object_type.as_str() {
            "blob" => {
                println!("fco blob: {:?}", path);
                Object::Blob { data: decompressed.to_vec(), path, status: ObjectStatus::Existed }
            },
            "tree" => {
                println!("fco tree: {:?}", path);
                let mut children: Vec<Object> = Vec::new();
                let lines = split_lines(decompressed);
                for line in lines {
                    let texts: Vec<String> = line.splitn(3, |c| &c == &' ').map(|s| s.to_string()).collect();
                    let child_type = texts.get(0).unwrap();
                    let child_hash = texts.get(1).unwrap();
                    let child_path = texts.get(2).unwrap();
                    let child_path = path.join(child_path);
                    let child = Object::from_compressed_obj(child_path, child_hash.to_string(), child_type.to_owned());
                    children.push(child);
                };
                Object::Tree { path, children, status: ObjectStatus::Existed }
            },
            _ => unreachable!()
        }
    }

    pub fn path(&self) -> PathBuf {
        let path = match self {
            Object::Blob {
                data: _,
                path,
                status: _,
            } => path,
            Object::Tree {
                path,
                children: _,
                status: _,
            } => path
        };
        PathBuf::from(path)
    }

    pub fn data(&self) -> Vec<u8> {
        match self {
            Object::Blob { data, path: _, status: _ } => {
                data.to_owned()
            },
            Object::Tree { path: _, children, status: _ } => {
                let mut v = Vec::new();
                for child in children {
                    let object_type = match modelize_entry(&child.path()) {
                        Some(Entry::Dir) => "tree",
                        Some(Entry::File) => "blob",
                        _ => unreachable!()
                    };
                    let line = format!("{} {} {}", object_type, child.hash(), child.path().file_name().unwrap().to_str().unwrap());
                    v.push(line);
                }
                v.join("\n").into_bytes()
            }
        }
    }

    pub fn to_show(&self) -> Option<String> {
        String::from_utf8(self.data()).ok()
    }

    pub fn compress(&self) -> Vec<u8> {
        compress_by_zlib(self.data().as_slice())
    }

    pub fn hash(&self) -> String {
        generate_hash(self.compress().as_slice())
    }

    pub fn write(&self) {
        let hash = self.hash();
        let path = format!("./.aequorea/objects/{}", hash);
        let path = Path::new(path.as_str());
        match self {
            Object::Tree {path: _, children, status: _} => {
                for child in children {
                    child.write();
                }
            },
            _ => ()
        }
        create_file(&path.to_path_buf(), self.compress().as_slice())
    }
}

impl ObjectDebug for Object {
    fn debug_print_path(&self) {
        match self {
            Object::Blob { data: _, path, status: _ } => {
                println!("blob: {:?}", path);
            },
            Object::Tree { path, children, status: _ } => {
                println!("tree: {:?}", path);
                for child in children {
                    child.debug_print_path();
                }
            }
        }
    }
}

fn modelize_entry(path: &PathBuf) -> Option<Entry> {
    if path.is_dir() {
        Some(Entry::Dir)
    } else if path.is_file() {
        Some(Entry::File)
    } else {
        None
    }
}

fn skip_entry(path: &PathBuf) -> bool {
    path.ends_with("target") | path.ends_with(".git") | path.ends_with(".aequorea")
}

fn current_path() -> PathBuf {
    Path::new(".").canonicalize().unwrap()
}

fn contain_child(parent: &PathBuf, child: &PathBuf) -> bool {
    child.starts_with(parent) & !(parent.starts_with(child))
}

pub fn first_add(path: PathBuf) {
    let object = Object::from_path(path);
    let path = object.path();
    let mut p = Path::new(&path).canonicalize().unwrap();
    let mut child = object;
    while contain_child(&current_path(), &p) {
        println!("{:?}", child.path());
        let parent_path = p.parent().unwrap();
        let parent = Object::Tree {
            path: parent_path.to_path_buf(),
            children: vec![child],
            status: ObjectStatus::Created,
        };
        p = parent_path.to_path_buf();
        child = parent;
    }
    let object = child;
    object.write();
}
