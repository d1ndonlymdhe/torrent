use std::net::{ToSocketAddrs, UdpSocket};
use rand::{thread_rng, Rng};

struct ConnectionRequest {
    protocol_id: i64,
    action: i32,
    transaction_id: i32,
}

impl ConnectionRequest {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let mut transaction_id = self.transaction_id;
        for _ in 0..4 {
            let r1 = (transaction_id % 0x10) as u8;
            transaction_id /= 0x10;
            let r2 = (transaction_id % 0x10) as u8;
            transaction_id /= 0x10;
            bytes.push(r2 * 0x10 + r1);
        }

        let mut action = self.action;
        for _ in 0..4 {
            let r1 = (action % 0x10) as u8;
            action /= 0x10;
            let r2 = (action % 0x10) as u8;
            action /= 0x10;
            bytes.push(r2 * 0x10 + r1);
        }

        let mut protocol_id = self.protocol_id;
        for _ in 0..8 {
            let r1 = (protocol_id % 0x10) as u8;
            protocol_id /= 0x10;
            let r2 = (protocol_id % 0x10) as u8;
            protocol_id /= 0x10;
            bytes.push(r2 * 0x10 + r1);
        }
        bytes.into_iter().rev().collect()
    }
}

pub enum ConnectionRequestAction {
    CONNECT
}
impl ConnectionRequestAction {
    fn get_code(&self) -> i32 {
        match self { ConnectionRequestAction::CONNECT => { 0 } }
    }
}

impl ConnectionRequest {
    pub fn new(action: ConnectionRequestAction) -> Self {
        let mut rng = thread_rng();
        let protocol_id = 0x41727101980; // magic constant
        let transaction_id = rng.gen_range(i32::MIN..i32::MAX);
        let action = action.get_code();
        ConnectionRequest {
            protocol_id,
            transaction_id,
            action,
        }
    }
}

pub fn connect() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let destination = "tracker.leechers-paradise.org:6969";
    let req = ConnectionRequest::new(ConnectionRequestAction::CONNECT).as_bytes();
    match socket.send_to(&req, destination.to_socket_addrs().unwrap().next().unwrap()) {
        Ok(bytes_sent) => {
            println!("sent {}", bytes_sent);
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
            action: 0,
            transaction_id: 0x1010,
        };
        let vec = vec![0x00, 0x00, 0x04, 0x17, 0x27, 0x10, 0x19, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10];
        assert_eq!(s.as_bytes(), vec);
    }
}