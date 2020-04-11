use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::Entry;
use crate::core::io::{compress_by_zlib, create_file, generate_hash, read_file_bytes, read_decoded, split_lines};
use crate::core::ignore::Ignore;

// #[derive(Debug)]
// pub enum ObjectStatus {
//     Created,
//     Existed,
//     Deleted,
// }

#[derive(Debug)]
pub enum Object {
    Blob {
        path: PathBuf,
        data: Vec<u8>
    },
    Tree {
        path: PathBuf,
        children: HashMap<PathBuf, Object>
    }
}

pub trait ObjectDebug {
    fn debug_print_path(&self);
    fn debug_print_detail(&self);
    fn debug_hash_list(&self) -> Vec<String>;
}

impl Object {
    pub fn from_path(path: PathBuf) -> Self {
        match Entry::modelize_entry(&path) {
            Some(Entry::File) => {
                let file = read_file_bytes(&path);
                Object::Blob {
                    path,
                    data: file
                }
            },
            Some(Entry::Dir) => {
                let mut hm = HashMap::new();
                let ignore = Ignore::new();
                for entry in fs::read_dir(&path).unwrap() {
                    let entry_path = entry.unwrap().path();
                    if ignore.for_path(&entry_path) {
                        continue;
                    }
                    let obj: Object = Object::from_path(entry_path);
                    hm.insert(obj.path(), obj);
                }
                Object::Tree {
                    path,
                    children: hm
                }
            },
            _ => unreachable!()
        }
    }

    pub fn from_compressed_obj(path: PathBuf, hash: String, obj_type: String) -> Self {
        let decompressed = read_decoded(&Self::obj_recorded_path(hash).to_path_buf());
        match obj_type.as_str() {
            "blob" => {
                Object::Blob { path, data: decompressed.to_vec() }
            },
            "tree" => {
                let mut children: HashMap<PathBuf, Object> = HashMap::new();
                let lines = split_lines(decompressed);
                for line in lines {
                    let texts: Vec<String> = line.splitn(3, |c| &c == &' ').map(|s| s.to_string()).collect();
                    let child_type = texts.get(0).unwrap();
                    let child_hash = texts.get(1).unwrap();
                    let child_path = texts.get(2).unwrap();
                    let child_path = path.join(child_path);
                    let child = Object::from_compressed_obj(child_path, child_hash.to_string(), child_type.to_owned());
                    children.insert(child.path(), child);
                };
                Object::Tree { path, children }
            },
            _ => unreachable!()
        }
    }

    pub fn obj_recorded_path(hash: String) -> PathBuf {
        let path = format!("./.aequorea/objects/{}", hash);
        let path = Path::new(&path);
        path.to_path_buf()
    }

    pub fn path(&self) -> PathBuf {
        let path = match self {
            Object::Blob {
                path,
                data: _
            } => path,
            Object::Tree {
                path,
                children: _
            } => path
        };
        PathBuf::from(path)
    }

    pub fn insert_child(&mut self, child: Object) -> Option<()> {
        match self {
            Object::Blob { path: _, data: _ } => {
                None
            },
            Object::Tree { path: _, children } => {
                children.insert(child.path(), child);
                Some(())
            }
        }
    }

    pub fn data(&self) -> Vec<u8> {
        match self {
            Object::Blob { path: _, data } => {
                data.to_owned()
            },
            Object::Tree { path: _, children } => {
                let mut v = Vec::new();
                for (path, child) in children {
                    let obj_type = match Entry::modelize_entry(&path) {
                        Some(Entry::Dir) => "tree",
                        Some(Entry::File) => "blob",
                        _ => unreachable!()
                    };
                    let line = format!("{} {} {}", obj_type, child.hash(), path.file_name().unwrap().to_str().unwrap());
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
        println!("<DEBUG> {:?}", self.path());
        // println!("<DEBUG> {:?}: {:?}", self.path(), self.to_show());
        let hash = self.hash();
        match self {
            Object::Tree { path: _, children } => {
                for (_, child) in children {
                    child.write();
                }
            },
            _ => ()
        }
        create_file(&Self::obj_recorded_path(hash).to_path_buf(), self.compress().as_slice())
    }
}

impl ObjectDebug for Object {
    fn debug_print_path(&self) {
        match self {
            Object::Blob { path, data: _ } => {
                println!("blob: {:?}", path);
            },
            Object::Tree { path, children } => {
                println!("tree: {:?}", path);
                for (_, child) in children {
                    child.debug_print_path();
                }
            }
        }
    }

    fn debug_print_detail(&self) {
        match self {
            Object::Blob { path, data } => {
                println!("blob: {:?}, {:?}", path, data);
            },
            Object::Tree { path, children } => {
                println!("tree: {:?}", path);
                for (_, child) in children {
                    child.debug_print_detail();
                }
            }
        }
    }

    fn debug_hash_list(&self) -> Vec<String> {
        match self {
            Object::Blob { path, data: _ } => {
                println!("chl: path {:?}", path);
                vec![self.hash()]
            },
            Object::Tree { path: _, children } => {
                let mut v: Vec<String> = Vec::new();
                for (_, child) in children {
                    v.extend(child.debug_hash_list());
                };
                println!("{:?}", v);
                v
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Object, ObjectDebug};
    use std::path::PathBuf;
    use std::collections::HashMap;

    fn setup() -> Object {
        let blob1 = Object::Blob { data: vec![1, 1, 1, 1], path: PathBuf::from("one") };
        let blob2 = Object::Blob { data: vec![2, 2, 2, 2], path: PathBuf::from("two") };
        let mut children = HashMap::new();
        children.insert(blob1.path(), blob1);
        children.insert(blob2.path(), blob2);
        let tree1 = Object::Tree { path: PathBuf::from("tree1"), children };
        tree1
    }

    #[test]
    fn test() {
        let mut root = setup();
        let blob3 = Object::Blob { data: vec![3, 3, 3, 3], path: PathBuf::from("three") };
        let mut children = HashMap::new();
        children.insert(blob3.path(), blob3);
        let tree2 = Object::Tree { path: PathBuf::from("tree2"), children };
        root.insert_child(tree2);
        root.debug_print_detail();
    }
}
