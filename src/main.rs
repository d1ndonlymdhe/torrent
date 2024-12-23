use std::fs::{read};
use std::net::{SocketAddr};
use sha1::{Digest, Sha1};
use socket2::{Domain, Protocol, Socket, Type};
use crate::bencode::{encode_bencode, parse_bencode, BDict, Bencode};
use crate::tracker::{announce, connect};

mod bencode;
mod str_utils;
mod tracker;

fn get_announce_list(info_dict: &BDict) -> Vec<String> {
    let mut announce_list: Vec<String> = Vec::new();
    let announce_url = info_dict.get("announce").unwrap_or_else(|| { panic!("No announce in file") });
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
fn get_info_hash(info_dict: BDict) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(encode_bencode(&Bencode::Dict(info_dict)));
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
            (get_announce_list(&info_dict), get_info_hash(info_dict))
        }
        _ => {
            panic!("Invalid torrent file")
        }
    };
    
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_only_v6(false).unwrap();
    
    let addr: SocketAddr = "[::]:0".parse().unwrap();
    socket.bind(&addr.into()).unwrap();
    
    let mut announce_response_list = Vec::new();

    for announce_url in announce_list {
        let response = connect(&announce_url, &socket);
        if response.is_err() {
            continue;
        }
        let connection_response = response.unwrap();
        let announce_response = announce(announce_url, connection_response.connection_id, connection_response.transaction_id, info_hash.clone(), &socket);
        if let Ok(announce_response) = announce_response {
            announce_response_list.push(announce_response);
        }
    }

    let mut peers = Vec::new();
    for announce_response in announce_response_list {
        for peer in announce_response.peers {
            if peer.0 != "0.0.0.0" && peer.1 >= 0 {
                peers.push(peer);
            }
        }
    }
    println!("Peers: {:?}", peers);
}
