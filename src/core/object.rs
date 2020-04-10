use std::fs;
use std::path::{Path, PathBuf};

use super::Entry;
use super::io::{compress_by_zlib, create_file, generate_hash, read_file_bytes, read_decoded, split_lines};

#[derive(Debug)]
pub enum ObjectStatus {
    Created,
    Existed,
    Deleted,
}

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
        match Entry::modelize_entry(&path) {
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
                    if Entry::exclude_entry(&entry_path) {
                        continue;
                    }
                    let obj: Object = Object::from_path(entry_path);
                    v.push(obj);
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

    pub fn obj_path(hash: String) -> PathBuf {
        let path = format!("./.aequorea/objects/{}", hash);
        let path = Path::new(&path);
        path.to_path_buf()
    }

    pub fn from_compressed_obj(path: PathBuf, hash: String, obj_type: String) -> Self {
        let decompressed = read_decoded(&Self::obj_path(hash).to_path_buf());
        match obj_type.as_str() {
            "blob" => {
                Object::Blob { data: decompressed.to_vec(), path, status: ObjectStatus::Existed }
            },
            "tree" => {
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
                    let obj_type = match Entry::modelize_entry(&child.path()) {
                        Some(Entry::Dir) => "tree",
                        Some(Entry::File) => "blob",
                        _ => unreachable!()
                    };
                    let line = format!("{} {} {}", obj_type, child.hash(), child.path().file_name().unwrap().to_str().unwrap());
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
        match self {
            Object::Tree {path: _, children, status: _} => {
                for child in children {
                    child.write();
                }
            },
            _ => ()
        }
        create_file(&Self::obj_path(hash).to_path_buf(), self.compress().as_slice())
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
