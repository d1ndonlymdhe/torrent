use std::mem::MaybeUninit;
use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;
use crate::tracker::utils::{parse_url};
use crate::tracker::types::{AnnounceRequest, AnnounceResponse, ConnectionRequest, ConnectionRequestAction, ConnectionResponse};

mod utils;
mod types;

pub fn connect(url: impl Into<String>, socket_v4: &UdpSocket, socket_v6: &UdpSocket) -> Result<ConnectionResponse, ()> {
    let max_tries = 1; // this should be 8 according to spec
    let try_coeff = 1; // this should be 15 according to spec

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

    let mut tries = 0;

    while tries < max_tries {
        let timeout = try_coeff * 2u64.pow(tries);
        tries += 1;
        println!("Sending connection request to {}", url);
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

pub fn announce(url: impl Into<String>, connection_id: i64, transaction_id: i32, info_hash: Vec<u8>, socket_v4: &UdpSocket, socket_v6: &UdpSocket) -> Result<AnnounceResponse, ()> {
    let max_tries = 1; // this should be 8 according to spec
    let try_coeff = 1; // this should be 15 according to spec
    let url = url.into();
    let (_, hostname, path) = parse_url(&url);
    let _: i16 = hostname.split(":").collect::<Vec<&str>>()[1].parse().unwrap();
    let request = AnnounceRequest::new(&connection_id, &transaction_id, info_hash.clone());

    let mut url_data_vec = vec![0x2, 0xc];
    url_data_vec.extend_from_slice(path.as_bytes());

    let mut request_bytes = request.to_req_bytes();
    request_bytes.extend(url_data_vec);

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
    println!("Sending announce request to {}, bytes = {}", url, bytes_sent);
    let mut buff = [0; 98];
    while tries < max_tries {
        let timeout = try_coeff * 2u64.pow(tries);
        tries += 1;
        let _ = socket.set_read_timeout(Some(Duration::new(timeout, 0)));
        if let Ok(len) = socket.recv(&mut buff) {
            //TODO: add checks
            let response = AnnounceResponse::from_bytes(&buff.to_vec(), len);
            return response;
        } else {
            println!("Could not receive announce request");
        }
    }
    println!("Could not announce to tracker {} in {} tries", url, tries);
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
        let vec = vec![0x00, 0x00, 0x04, 0x17, 0x27, 0x10, 0x19, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10];
        assert_eq!(s.to_req_bytes(), vec);
    }
}