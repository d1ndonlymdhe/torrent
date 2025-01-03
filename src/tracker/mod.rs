use crate::bencode::parse_bencode;
use crate::tracker::types::{
    AnnounceRequest, AnnounceResponse, ConnectionRequest, ConnectionRequestAction,
    ConnectionResponse,
};
use crate::tracker::utils::parse_url;
use percent_encoding::{percent_encode, CONTROLS, NON_ALPHANUMERIC};
use reqwest::Url;
use std::mem::MaybeUninit;
use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;

pub mod types;
mod utils;

pub fn connect(
    url: impl Into<String>,
    socket_v4: &UdpSocket,
    socket_v6: &UdpSocket,
) -> Result<ConnectionResponse, ()> {
    let max_tries = 1; // this should be 8 according to spec
    let try_coeff = 2; // this should be 15 according to spec

    let url = url.into();
    let (protocol, hostname, path) = parse_url(&url);

    if protocol != "udp" {
        println!("Only UDP tracker supported");
        return Err(());
    }

    let mut url_data_vec = vec![0x2, 0xc];
    url_data_vec.extend_from_slice(path.as_bytes());

    let request = ConnectionRequest::new(ConnectionRequestAction::CONNECT);
    let mut req_bytes = request.to_req_bytes();
    req_bytes.extend(url_data_vec);
    let dest_addr = hostname.to_socket_addrs();
    if dest_addr.is_err() {
        println!("Cannot resolve hostname");
        return Err(());
    }
    let dest_addr = dest_addr.unwrap().next();
    if dest_addr.is_none() {
        println!("Cannot resolve hostname");
        return Err(());
    }
    let dest_addr = dest_addr.unwrap();
    let socket = if dest_addr.is_ipv6() {
        println!("Is v6");
        socket_v6
    } else {
        println!("Is v4");
        socket_v4
    };

    let mut tries = 0;

    while tries < max_tries {
        let timeout = try_coeff * 2u64.pow(tries);
        tries += 1;
        let send_result = socket.send_to(&req_bytes, dest_addr);
        match send_result {
            Ok(_) => {
                let mut buf = [0; 20];
                let _ = socket.set_read_timeout(Some(Duration::new(timeout, 0)));
                let res_size = socket_v4.recv(&mut buf);
                if let Ok(res_size) = res_size {
                    if res_size >= 16 {
                        //TODO: add checks
                        let response = ConnectionResponse::from_res_bytes(&buf);
                        return response;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed {}", e);
            }
        }
    }
    println!("Could not connect to tracker {} in {} tries", url, tries);
    Err(())
}

pub fn announce(
    url: impl Into<String>,
    connection_id: i64,
    info_hash: Vec<u8>,
    socket_v4: &UdpSocket,
    socket_v6: &UdpSocket,
) -> Result<AnnounceResponse, ()> {
    let max_tries = 1; // this should be 8 according to spec
    let try_coeff = 2; // this should be 15 according to spec
    let url = url.into();
    let (_, hostname, path) = parse_url(&url);
    let _: i16 = hostname.split(":").collect::<Vec<&str>>()[1]
        .parse()
        .unwrap();
    let request = AnnounceRequest::new(&connection_id, info_hash.clone());

    let mut url_data_vec = vec![0x2, 0xc];
    url_data_vec.extend_from_slice(path.as_bytes());

    let mut request_bytes = request.to_req_bytes();
    if !path.is_empty() {
        request_bytes.extend(url_data_vec);
        request_bytes.extend(vec![0x1, 0x1, 0x0]);
    }
    let dest_addr = hostname.to_socket_addrs();
    if dest_addr.is_err() {
        return Err(());
    }
    let dest_addr = dest_addr.unwrap().next();
    if dest_addr.is_none() {
        return Err(());
    }
    let dest_addr = dest_addr.unwrap();
    let socket = if dest_addr.is_ipv6() {
        socket_v6
    } else {
        socket_v4
    };
    let bytes_sent = socket.send_to(request_bytes.as_slice(), dest_addr).unwrap();

    let mut tries = 0;
    let mut buff = [0; 1024];
    while tries < max_tries {
        let timeout = try_coeff * 2u64.pow(tries);
        tries += 1;
        let _ = socket.set_read_timeout(Some(Duration::new(timeout, 0)));
        if let Ok(len) = socket.recv(&mut buff) {
            //TODO: add checks
            let response = AnnounceResponse::from_bytes(&buff, len);
            return response;
        } else {
            println!("Could not receive announce request");
        }
    }
    println!("Could not announce to tracker {} in {} tries", url, tries);
    Err(())
}

pub fn announce_http(
    url: impl Into<String>,
    announce_request: AnnounceRequest,
) -> Result<AnnounceResponse, ()> {
    let url = url.into();
    let AnnounceRequest {
        info_hash,
        peer_id,
        ip_address,
        port,
        uploaded,
        downloaded,
        left,
        event,
        ..
    } = announce_request;
    let encoding_set = CONTROLS;
    let info_hash = percent_encode(&info_hash, encoding_set).to_string();
    let url = Url::parse_with_params(
        format!("{}?info_hash={}", &url, info_hash).as_str(),
        &[
            ("peer_id", &"-qb1001-abcdfhijklolpl".to_string()),
            ("ip", &ip_address.to_string()),
            ("port", &port.to_string()),
            ("uploaded", &uploaded.to_string()),
            ("downloaded", &downloaded.to_string()),
            ("left", &left.to_string()),
            ("event", &"none".to_string()),
        ],
    )
    .unwrap();
    println!("{}", url.as_str());
    let response = reqwest::blocking::get(url.as_str()).unwrap();

    println!("{:#?}", response.text());
    Err(())
}

#[cfg(test)]
mod tracker_tests {
    use super::*;
    #[test]
    fn test_bytes() {
        let s = ConnectionRequest {
            protocol_id: 0x41727101980,
            action: ConnectionRequestAction::CONNECT,
            transaction_id: 0x1010,
        };
        let vec = vec![
            0x00, 0x00, 0x04, 0x17, 0x27, 0x10, 0x19, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x10,
        ];
        assert_eq!(s.to_req_bytes(), vec);
    }
}
