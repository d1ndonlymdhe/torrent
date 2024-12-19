use std::fs::{read};
use crate::bencode::parse_bencode;
use crate::tracker::connect;

mod bencode;
mod str_utils;
mod tracker;

fn main() {
    connect();
    // let content = read("test.torrent").unwrap();
    // 
    // println!("{:#?}", parse_bencode(&content));
}
