use std::net::{ToSocketAddrs, UdpSocket};
use std::time::Duration;
use rand::{thread_rng, Rng};
use crate::str_utils::sub_arr;

#[derive(Debug)]
struct ConnectionRequest {
    protocol_id: i64,
    action: ConnectionRequestAction,
    transaction_id: i32,
}
impl ConnectionRequest {
    pub fn new(action: ConnectionRequestAction) -> Self {
        let mut rng = thread_rng();
        let protocol_id = 0x41727101980; // magic constant
        let transaction_id = rng.gen_range(0..i32::MAX);
        // let action = action.get_code();
        ConnectionRequest {
            protocol_id,
            transaction_id,
            action,
        }
    }
    fn to_req_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(int_to_bytes(self.transaction_id as i128, 4));
        bytes.extend(int_to_bytes(self.action.get_code() as i128, 4));
        bytes.extend(int_to_bytes(self.protocol_id as i128, 8));
        bytes.into_iter().rev().collect()
    }
}

#[derive(Debug)]
pub enum ConnectionRequestAction {
    CONNECT,
    ANNOUNCE,
}
impl ConnectionRequestAction {
    fn get_code(&self) -> i32 {
        match self {
            ConnectionRequestAction::CONNECT => { 0 }
            ConnectionRequestAction::ANNOUNCE => { 1 }
        }
    }
    fn from_code(code: i32) -> ConnectionRequestAction {
        match code {
            0 => {
                ConnectionRequestAction::CONNECT
            }
            1 => {
                ConnectionRequestAction::ANNOUNCE
            }
            _ => {
                panic!("Invalid action code {}", code)
            }
        }
    }
}

#[derive(Debug)]
pub struct ConnectionResponse {
    pub action: ConnectionRequestAction,
    pub transaction_id: i32,
    pub connection_id: i64,
}

impl ConnectionResponse {
    fn from_res_bytes(bytes: &[u8]) -> Self {
        let action_bytes = &bytes[0..4];
        let transaction_id_bytes = &bytes[4..8];
        let connection_id_bytes = &bytes[8..16];
        let action = ConnectionRequestAction::from_code(bytes_to_int(action_bytes) as i32);
        let transaction_id = bytes_to_int(transaction_id_bytes) as i32;
        let connection_id = bytes_to_int(connection_id_bytes) as i64;
        ConnectionResponse {
            action,
            transaction_id,
            connection_id,
        }
    }
}

fn int_to_bytes(int: i128, size: usize) -> Vec<u8> {
    let mut int = int;
    let mut bytes = Vec::new();
    for _ in 0..size {
        let r = (int % 0x100) as u8;
        int /= 0x100;
        bytes.push(r);
    }
    bytes
}


struct AnnounceRequest {
    connection_id: i64,
    action: ConnectionRequestAction,
    transaction_id: i32,
    // this needs to be 20 bytes
    info_hash: Vec<u8>,
    //this needs to be 20 bytes
    peer_id: Vec<u8>,
    downloaded: i64,
    left: i64,
    uploaded: i64,
    event: i32,
    ip_address: i32,
    key: i32,
    num_want: i32,
    port: i16,
}

#[derive(Debug)]
struct AnnounceResponse {
    action: ConnectionRequestAction,
    transaction_id: i32,
    interval: i32,
    leechers: i32,
    seeders: i32,
    // IP address and TCP port
    peers: Vec<(i32, i16)>,
}

impl AnnounceResponse {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        let action_bytes = &bytes[0..4];
        let transaction_id_bytes = &bytes[4..8];
        let interval_bytes = &bytes[8..12];
        let leechers_bytes = &bytes[12..16];
        let seeders_bytes = &bytes[16..20];
        let mut offset = 20;
        let mut peers = Vec::new();
        while offset < bytes.len() {
            let new_slice = sub_arr(bytes.to_vec(), offset, offset + 6);
            let ip_bytes = &new_slice[0..4];
            let port_bytes = &new_slice[4..6];
            peers.push((bytes_to_int(ip_bytes) as i32, bytes_to_int(port_bytes) as i16));
            offset += 6;
        }
        AnnounceResponse {
            action: ConnectionRequestAction::from_code(bytes_to_int(action_bytes) as i32),
            transaction_id: bytes_to_int(transaction_id_bytes) as i32,
            interval: bytes_to_int(interval_bytes) as i32,
            leechers: bytes_to_int(leechers_bytes) as i32,
            seeders: bytes_to_int(seeders_bytes) as i32,
            peers,
        }
    }
}

impl AnnounceRequest {
    fn new(connection_id: &i64, transaction_id: &i32, info_hash: Vec<u8>) -> Self {
        let mut id = b"-PC0001-".to_vec();
        let mut rng = thread_rng();
        let id_num = rng.gen_range(0..0xFFF);
        let key = rng.gen_range(0..0xFFFFFF);
        id.extend(int_to_bytes(id_num, 12));
        AnnounceRequest {
            connection_id: *connection_id,
            action: ConnectionRequestAction::ANNOUNCE,
            transaction_id: *transaction_id,
            info_hash,
            peer_id: id,
            downloaded: 0,
            left: 0,
            uploaded: 0,
            event: 0,
            ip_address: 0,
            key,
            num_want: -1,
            port: 0,
        }
    }

    fn to_req_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(int_to_bytes(self.port as i128, 2));
        bytes.extend(int_to_bytes(self.num_want as i128, 4));
        bytes.extend(int_to_bytes(self.key as i128, 4));
        bytes.extend(int_to_bytes(self.ip_address as i128, 4));
        bytes.extend(int_to_bytes(self.event as i128, 4));
        bytes.extend(int_to_bytes(self.uploaded as i128, 8));
        bytes.extend(int_to_bytes(self.left as i128, 8));
        bytes.extend(int_to_bytes(self.downloaded as i128, 8));
        bytes.extend(&self.peer_id.clone().into_iter().rev().collect::<Vec<u8>>());
        bytes.extend(&self.info_hash.clone().into_iter().rev().collect::<Vec<u8>>());
        bytes.extend(int_to_bytes(self.transaction_id as i128, 4));
        bytes.extend(int_to_bytes(self.action.get_code() as i128, 4));
        bytes.extend(int_to_bytes(self.connection_id as i128, 8));
        bytes.into_iter().rev().collect()
    }
}

fn bytes_to_int(bytes: &[u8]) -> i128 {
    let mut num = 0;
    for (idx, byte) in bytes.iter().rev().enumerate() {
        let byte = *byte as i128;
        num += byte * 0x100i128.pow(idx as u32);
    }
    num
}


fn parse_url(url: impl Into<String>) -> (String, String, String) {
    let url = url.into();
    let mut protocal = Vec::new();
    let mut host = Vec::new();
    let mut path = Vec::new();
    let mut flag = 0;
    let mut idx = 0;
    let url_chars = url.split("").collect::<Vec<&str>>();
    while idx <= url.len() {
        let char = url_chars[idx];
        if char.eq("") {
            idx += 1;
            continue;
        }
        if flag == 0 {
            if char.ne(":") {
                protocal.push(char);
                idx += 1;
            } else {
                flag = 1;
                idx += 3;
                continue;
            }
        }
        if flag == 1 {
            if char.ne("/") {
                host.push(char);
                idx += 1;
            } else {
                flag = 2;
                idx += 1;
                continue;
            }
        }
        if flag == 2 {
            path.push(char);
            idx += 1;
        }
    }
    (protocal.into_iter().collect(), host.into_iter().collect(), path.into_iter().collect())
}

pub fn connect(url: impl Into<String>, socket: &UdpSocket) -> Result<ConnectionResponse, ()> {
    let max_tries = 1;
    let try_coeff = 1;

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
    let dest_addr = hostname.to_socket_addrs().unwrap().next().unwrap();
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
                let res_size = socket.recv(&mut buf);
                if let Ok(res_size) = res_size {
                    if res_size >= 16 {
                        let response = ConnectionResponse::from_res_bytes(&buf);
                        return Ok(response);
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

pub fn announce(url: impl Into<String>, connection_id: i64, transaction_id: i32, info_hash: Vec<u8>, socket: &UdpSocket) {
    // socket.set_write_timeout(None).unwrap();

    let url = url.into();
    let (protocol, hostname, path) = parse_url(&url);
    let port: i16 = hostname.split(":").collect::<Vec<&str>>()[1].parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:43792").unwrap();
    let request = AnnounceRequest::new(&connection_id, &transaction_id, info_hash.clone());

    let mut url_data_vec = vec![0x2, 0xc];
    url_data_vec.extend_from_slice(path.as_bytes());

    let mut request_bytes = request.to_req_bytes();
    request_bytes.extend(url_data_vec);

    let destination_addr = hostname.to_socket_addrs().unwrap().next().unwrap();
    let bytes_sent = socket.send_to(request_bytes.as_slice(), destination_addr).unwrap();
    println!("Sending announce request to {}, bytes = {}", url, bytes_sent);
    let mut buff = [0; 98];
    if let Ok(res) = socket.recv(&mut buff) {
        eprintln!("Received announce request from {}, bytes = {}", destination_addr, res);
        let response = AnnounceResponse::from_bytes(buff.to_vec());
        println!("Received announce response from {}, bytes = {:?}", destination_addr, response);
    } else {
        println!("Could not receive announce request");
    }
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