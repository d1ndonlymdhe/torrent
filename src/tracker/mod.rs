use std::arch::x86_64::_mm256_i32gather_epi32;
use std::net::{ToSocketAddrs, UdpSocket};
use rand::{thread_rng, Rng};

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

        let mut transaction_id = self.transaction_id;
        for _ in 0..4 {
            let r = (transaction_id % 0x100) as u8;
            transaction_id /= 0x100;
            bytes.push(r);
        }

        let mut action = self.action.get_code();
        for _ in 0..4 {
            let r = (action % 0x100) as u8;
            action /= 0x100;
            bytes.push(r);
        }

        let mut protocol_id = self.protocol_id;
        for _ in 0..8 {
            let r = (protocol_id % 0x100) as u8;
            protocol_id /= 0x100;
            bytes.push(r);
        }
        bytes.into_iter().rev().collect()
    }
}

#[derive(Debug)]
pub enum ConnectionRequestAction {
    CONNECT
}
impl ConnectionRequestAction {
    fn get_code(&self) -> i32 {
        match self { ConnectionRequestAction::CONNECT => { 0 } }
    }
    fn from_code(code: i32) -> ConnectionRequestAction {
        match code {
            0 => {
                ConnectionRequestAction::CONNECT
            }
            _ => {
                panic!("Invalid action code {}", code)
            }
        }
    }
}

#[derive(Debug)]
struct ConnectionResponse {
    action: ConnectionRequestAction,
    transaction_id: i32,
    connection_id: i64,
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

fn bytes_to_int(bytes: &[u8]) -> i128 {
    let mut num = 0;
    for (idx, byte) in bytes.iter().rev().enumerate() {
        let byte = *byte as i128;
        num += byte * 0x100i128.pow(idx as u32);
    }
    num
}


pub fn connect() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let destination = "tracker.opentrackr.org:1337";
    let request = ConnectionRequest::new(ConnectionRequestAction::CONNECT);
    let req_bytes = request.to_req_bytes();
    let dest_addr = destination.to_socket_addrs().unwrap().next().unwrap();
    match socket.send_to(&req_bytes, dest_addr) {
        Ok(bytes_sent) => {
            println!("sent {}", bytes_sent);
            let mut buf = [0; 20];
            let res_size = socket.recv(&mut buf).unwrap();
            if res_size == 16 {
                let response = ConnectionResponse::from_res_bytes(&buf);
                println!("{:#?}", response);
            }
        }
        Err(e) => {
            eprintln!("Failed {}", e);
        }
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