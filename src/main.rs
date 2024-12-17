use std::fs::{read};
use crate::bencode::parse_bencode;

mod bencode;
mod str_utils;

fn main() {
    let content = read("test.torrent").unwrap();

    println!("{:#?}", parse_bencode(&content));
}
