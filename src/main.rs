use std::fs::{read};
use std::net::{SocketAddr, UdpSocket};
use sha1::{Digest, Sha1};
use crate::bencode::{encode_bencode, parse_bencode, BDict, Bencode};
use crate::tracker::{announce, announce_http, connect};
use crate::tracker::types::AnnounceRequest;

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

    let socket_v4 = UdpSocket::bind("0.0.0.0:0").unwrap();
    let socket_v6 = UdpSocket::bind("[::]:0").unwrap();
    println!("{:?}",info_hash.clone());
    let mut announce_response_list = Vec::new();
    announce_http("http://tracker.opentrackr.org:1337/announce", AnnounceRequest::new(
        &0,
        info_hash.clone(),
    )).unwrap();


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
        let announce_response = announce(announce_url, connection_response.connection_id, info_hash.clone(), &socket_v4, &socket_v6);
        if announce_response.is_err() {
            println!("ANNOUNCE ERROR");
            println!("----");
            continue;
        }
        println!("ANNOUNCE SUCCESS");
        announce_response_list.push(announce_response.unwrap());
        println!("-----");
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
