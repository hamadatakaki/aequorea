use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use libflate::zlib::{Decoder, Encoder};

use crate::exit_process_with_error;

pub fn generate_hash(source: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.input(source);
    let s = hasher.result_str();
    String::from(s)
}

pub fn compress_by_zlib(source: &[u8]) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new())
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to make new encoder instance: {}", e));
    encoder
        .write_all(source)
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to write source data: {}", e));
    encoder
        .finish()
        .into_result()
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to encode: {}", e))
}

pub fn decompress_by_zlib(source: &[u8]) -> Vec<u8> {
    let mut decoder = Decoder::new(source)
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to make new decoder instance: {}", e));
    let mut buffer = Vec::new();
    decoder
        .read_to_end(&mut buffer)
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to decode: {}", e));
    buffer
}

pub fn create_file(path: &PathBuf, body: &[u8]) {
    let mut file =
        fs::File::create(path).unwrap_or_else(|e| exit_process_with_error!(1, "Failed to create file: {}", e));
    let size = file
        .write(body)
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to write into file: {}", e));
    assert_eq!(body.len(), size);
}

pub fn create_encoded(path: &PathBuf, body: &[u8]) {
    let encoded = compress_by_zlib(body);
    create_file(path, &encoded)
}

pub fn read_file_str(path: &PathBuf) -> String {
    let mut file =
        fs::File::open(path).unwrap_or_else(|e| exit_process_with_error!(1, "Failed to open file: {}", e));
    let mut text = String::new();
    file.read_to_string(&mut text)
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to read file: {}", e));
    text
}

pub fn read_file_bytes(path: &PathBuf) -> Vec<u8> {
    let mut file =
        fs::File::open(path).unwrap_or_else(|e| exit_process_with_error!(1, "Failed to open file: {}", e));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .unwrap_or_else(|e| exit_process_with_error!(1, "Failed to read file: {}", e));
    buffer
}

pub fn read_lines(path: &PathBuf) -> Vec<String> {
    let buffer = read_file_bytes(path);
    split_lines(buffer)
}

pub fn read_last_line(path: &PathBuf) -> String {
    let buffer = read_file_bytes(path);
    extract_last_line(buffer)
}

pub fn read_decoded(path: &PathBuf) -> Vec<u8> {
    let buffer = read_file_bytes(path);
    decompress_by_zlib(buffer.as_slice())
}

pub fn split_lines(buffer: Vec<u8>) -> Vec<String> {
    let lines: Vec<String> = buffer
        .split(|b| b == &10)
        .map(|s| String::from_utf8(s.to_vec()).unwrap())
        .collect();
    lines
}

fn extract_last_line(buffer: Vec<u8>) -> String {
    let last = buffer
        .split(|b| b == &10)
        .into_iter()
        .map(|s| String::from_utf8(s.to_vec()).unwrap())
        .last()
        .unwrap();
    last
}

#[cfg(test)]
mod tests {
    use super::{extract_last_line, split_lines};

    #[test]
    fn test_split_lines() {
        let bytes = vec![
            // 123 456 お腹空いた\n
            49, 50, 51, 32, 52, 53, 54, 32, 227, 129, 138, 232, 133, 185, 231, 169, 186, 227, 129,
            132, 227, 129, 159, 10, // 234 567 :divide: 10/0
            50, 51, 52, 32, 53, 54, 55, 32, 58, 100, 105, 118, 105, 100, 101, 58, 32, 49, 48, 47,
            48,
        ];
        let lines = split_lines(bytes);
        let s1 = String::from("123 456 お腹空いた");
        let s2 = String::from("234 567 :divide: 10/0");
        assert_eq!(vec![s1, s2], lines);
    }

    #[test]
    fn test_extract_last_line() {
        let bytes = vec![
            // 123 456 お腹空いた\n
            49, 50, 51, 32, 52, 53, 54, 32, 227, 129, 138, 232, 133, 185, 231, 169, 186, 227, 129,
            132, 227, 129, 159, 10, // 234 567 :divide: 10/0
            50, 51, 52, 32, 53, 54, 55, 32, 58, 100, 105, 118, 105, 100, 101, 58, 32, 49, 48, 47,
            48,
        ];
        let line = extract_last_line(bytes);
        let s = String::from("234 567 :divide: 10/0");
        assert_eq!(s, line);
    }

    #[test]
    fn test_extract_last_line2() {
        let bytes = vec![];
        let line = extract_last_line(bytes);
        assert!(line.is_empty());
    }
}
