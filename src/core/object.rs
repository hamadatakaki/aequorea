use std::fs;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::exit_process_with_error;
use crate::core::{current_path, Entry};
use crate::core::io::{compress_by_zlib, create_file, generate_hash, read_file_bytes, read_decoded, split_lines};
use crate::core::ignore::Ignore;

// #[derive(Debug)]
// pub enum ObjectStatus {
//     Created,
//     Existed,
//     Deleted,
// }

pub enum ObjectType {
    Blob,
    Tree
}

impl ObjectType {
    pub fn from_str(string: &str) -> Self {
        if string == "tree" {
            Self::Tree
        } else if string == "blob" {
            Self::Blob
        } else {
            exit_process_with_error!(1, "The type does not exist: {}", string);
        }
    }
}

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

    pub fn from_compressed_obj(path: PathBuf, hash: String, obj_type: ObjectType) -> Self {
        let decompressed = read_decoded(&Self::obj_recorded_path(hash).to_path_buf());
        match obj_type {
            ObjectType::Blob => {
                Object::Blob { path, data: decompressed.to_vec() }
            },
            ObjectType::Tree => {
                let mut children: HashMap<PathBuf, Object> = HashMap::new();
                let lines = split_lines(decompressed);
                for line in lines {
                    let texts: Vec<String> = line.splitn(3, |c| &c == &' ').map(|s| s.to_string()).collect();
                    let child_type = texts.get(0).unwrap();
                    let child_type = ObjectType::from_str(child_type);
                    let child_hash = texts.get(1).unwrap();
                    let child_path = texts.get(2).unwrap();
                    let child_path = path.join(child_path);
                    let child = Object::from_compressed_obj(child_path, child_hash.to_string(), child_type);
                    children.insert(child.path(), child);
                };
                Object::Tree { path, children }
            }
        }
    }

    pub fn obj_recorded_path(hash: String) -> PathBuf {
        let mut path = current_path();
        path.push(".aequorea/objects");
        path.push(hash);
        path
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

    pub fn all_contained_hash(&self) -> Vec<String> {
        match self {
            Object::Blob { path: _, data: _ } => {
                vec![self.hash()]
            },
            Object::Tree { path: _, children } => {
                let mut v = vec![self.hash()];
                for (_, child) in children {
                    v.extend(child.all_contained_hash());
                };
                v
            }
        }
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

    pub fn replace(&mut self, path: PathBuf, obj: Self) -> Result<(), String> {
        // pathとselfのパスの属関係を確認する
        //     /root/hogeに対し/root/foo/barをreplaceしようとしていたらErrを返す、みたいな
        // childrenにpathがあるか確認する
        // あれば置き換え、なければchildren内のTreeのreplaeを呼び出す
        if &path.starts_with(self.path()) & !(self.path().starts_with(&path)) {
            Err(String::new())
        } else {
            // match self {
            //     Object::Tree { path: _, children } => {
            //         match children.remove(&path) {
            //             Some(_) => {
            //                 children.insert(path, obj);
            //             },
            //             _ => ()
            //         }
            //     },
            //     _ => ()
            // }
            Ok(())
        }
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
}

#[cfg(test)]
mod tests {
    use super::{Object, ObjectDebug};
    use std::path::PathBuf;
    use std::collections::HashMap;

    fn setup() -> Object {
        let blob1 = Object::Blob { data: vec![1, 1, 1, 1], path: PathBuf::from("/first/one") };
        let blob2 = Object::Blob { data: vec![2, 2, 2, 2], path: PathBuf::from("/first/two") };
        let blob3 = Object::Blob { data: vec![3, 3, 3, 3], path: PathBuf::from("/second/three") };
        let blob4 = Object::Blob { data: vec![4, 4, 4, 4], path: PathBuf::from("/secondfour") };
        let blob5 = Object::Blob { data: vec![5, 5, 5, 5], path: PathBuf::from("/five") };

        let mut children1 = HashMap::new();
        children1.insert(blob1.path(), blob1);
        children1.insert(blob2.path(), blob2);
        let mut children2 = HashMap::new();
        children2.insert(blob3.path(), blob3);
        children2.insert(blob4.path(), blob4);

        let tree1 = Object::Tree { path: PathBuf::from("/first"), children: children1 };
        let tree2 = Object::Tree { path: PathBuf::from("/second"), children: children2 };

        let mut root = HashMap::new();
        root.insert(tree1.path(), tree1);
        root.insert(tree2.path(), tree2);
        root.insert(blob5.path(), blob5);

        let root = Object::Tree { path: PathBuf::from("/"), children: root };

        root
    }

    // #[test]
    // fn test_insert() {
    //     let mut root = setup();
    //     let blob3 = Object::Blob { data: vec![3, 3, 3, 3], path: PathBuf::from("three") };
    //     let mut children = HashMap::new();
    //     children.insert(blob3.path(), blob3);
    //     let tree2 = Object::Tree { path: PathBuf::from("tree2"), children };
    //     root.insert_child(tree2);
    //     root.debug_print_detail();
    // }

    #[test]
    fn test_replace() {
        let mut root = setup();
        let new_blob3 = Object::Blob { data: vec![4, 5, 6, 7], path: PathBuf::from("/second/three") };
        root.replace(new_blob3.path(), new_blob3).unwrap();

        root.debug_print_detail();
    }
}
