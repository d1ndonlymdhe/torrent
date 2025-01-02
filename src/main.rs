use crate::bencode::{encode_bencode, parse_bencode, BDict, Bencode};
use crate::tracker::{announce, connect};
use sha1::{Digest, Sha1};
use std::fs::read;
use std::net::{UdpSocket};

mod bencode;
mod str_utils;
mod tracker;

fn get_announce_list(info_dict: &BDict) -> Vec<String> {
    let mut announce_list: Vec<String> = Vec::new();
    let announce_url = info_dict
        .get("announce")
        .unwrap_or_else(|| panic!("No announce in file"));
    if let Bencode::Str(announce_url) = announce_url {
        announce_list.push(String::from_utf8(announce_url.to_vec()).unwrap())
    }
    let announce_list_data = info_dict.get("announce-list");
    if announce_list_data.is_some() {
        let announce_list_data = announce_list_data.unwrap();
        if let Bencode::List(announce_list_data) = announce_list_data {
            for announce_url in announce_list_data {
                if let Bencode::List(announce_url) = announce_url {
                    for announce_url in announce_url {
                        if let Bencode::Str(announce_url) = announce_url {
                            announce_list.push(String::from_utf8(announce_url.to_vec()).unwrap())
                        }
                    }
                }
            }
        }
    }
    announce_list
}
fn get_info_hash(info_dict: &BDict) -> Vec<u8> {
    let mut hasher = Sha1::new();
    let encoded = encode_bencode(&Bencode::Dict(info_dict.clone()));
    hasher.update(&encoded);
    hasher.finalize().as_slice().to_vec()
}

fn main() {
    let content = read("test.torrent").unwrap();
    let parsed = parse_bencode(&content);

    if parsed.is_err() {
        panic!("Invalid torrent file")
    }

    let file_data = parsed.unwrap().data;
    let (announce_list, info_hash) = match file_data {
        Bencode::Dict(info_dict) => {
            let announce_list = get_announce_list(&info_dict);
            let info_dict = info_dict.get("info").expect("No info in file");
            if let Bencode::Dict(info_dict) = info_dict {
                (announce_list, get_info_hash(info_dict))
            } else {
                panic!("Invalid torrent file")
            }
        }
        _ => {
            panic!("Invalid torrent file")
        }
    };
    let mut announce_response_list = Vec::new();

    let socket_v4 = UdpSocket::bind("0.0.0.0:0").unwrap();
    let socket_v6 = UdpSocket::bind("[::]:0").unwrap();

    for announce_url in announce_list {
        println!("URL: {}", announce_url);
        let connect_response = connect(&announce_url, &socket_v4, &socket_v6);
        if connect_response.is_err() {
            println!("CONNECT ERROR");
            println!("----");
            continue;
        }
        println!("CONNECT SUCCESS");
        let connection_response = connect_response.unwrap();
        let announce_response = announce(
            announce_url,
            connection_response.connection_id,
            info_hash.clone(),
            &socket_v4,
            &socket_v6,
        );
        if announce_response.is_err() {
            println!("ANNOUNCE ERROR");
            println!("----");
            continue;
        }
        println!("ANNOUNCE SUCCESS");
        announce_response_list.push(announce_response.unwrap());
        println!("-----");
    }

    // let mut peers = HashSet::new();
    // for announce_response in announce_response_list {
    //     for peer in announce_response.peers {
    //         if peer.0 != "0.0.0.0" && peer.1 >= 0 {
    //             peers.insert(peer);
    //         }
    //     }
    // }
    println!("Peers: {:?}", announce_response_list);
    println!("Peers count: {}", announce_response_list.len());
}
