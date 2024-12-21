use std::fs::{read};
use crate::bencode::{parse_bencode, Bencode};
use crate::tracker::connect;

mod bencode;
mod str_utils;
mod tracker;

fn main() {
    let content = read("test.torrent").unwrap();
    let parsed = parse_bencode(&content);
    if parsed.is_err() {
        panic!("Invalid torrent file")
    }
    let file_data = parsed.unwrap().data;
    let mut announce_list = Vec::new();
    if let Bencode::Dict(main_dict) = file_data {
        let announce_url = main_dict.get("announce").unwrap_or_else(|| { panic!("No announce in file") });
        if let Bencode::Str(announce_url) = announce_url {
            announce_list.push(String::from_utf8(announce_url.to_vec()).unwrap())
        }
        let announce_list_data = main_dict.get("announce-list");
        if announce_list_data.is_some() {
            let announce_list_data = announce_list_data.unwrap();
            if let Bencode::List(announce_list_data) = announce_list_data {
                for announce_url in announce_list_data {
                    if let Bencode::List(announce_url) = announce_url {
                        for announce_URL in announce_url {
                            if let Bencode::Str(announce_url) = announce_URL {
                                announce_list.push(String::from_utf8(announce_url.to_vec()).unwrap())
                            }
                        }
                    }
                }
            }
        }
    }
    for announce_url in announce_list {
        let connection_response = connect(&announce_url);
        if connection_response.is_err() {
            continue;
        }
        let connection_response = connection_response.unwrap();
        println!("{}", announce_url);
        println!("{:?}", connection_response);
        break;
    }
}
