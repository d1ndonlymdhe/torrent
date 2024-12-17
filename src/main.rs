use std::fs::read_to_string;
use crate::bencode::parse_bencode;

mod bencode;
mod str_utils;

fn main() {
    let content = read_to_string("test.torrent").unwrap();

    println!("{:#?}", parse_bencode(&content.as_bytes().to_vec()));
}
