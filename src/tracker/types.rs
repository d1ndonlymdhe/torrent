use rand::{thread_rng, Rng};
use crate::str_utils::sub_arr;
use crate::tracker::utils::{bytes_to_int, int_to_bytes};

#[derive(Debug)]
pub struct ConnectionRequest {
    pub(crate) protocol_id: i64,
    pub(crate) action: ConnectionRequestAction,
    pub(crate) transaction_id: i32,
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
    pub(crate) fn to_req_bytes(&self) -> Vec<u8> {
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
    fn from_code(code: i32) -> Result<ConnectionRequestAction, ()> {
        match code {
            0 => {
                Ok(ConnectionRequestAction::CONNECT)
            }
            1 => {
                Ok(ConnectionRequestAction::ANNOUNCE)
            }
            _ => {
                println!("Invalid action code {}", code);
                Err(())
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
    pub(crate) fn from_res_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let action_bytes = &bytes[0..4];
        let transaction_id_bytes = &bytes[4..8];
        let connection_id_bytes = &bytes[8..16];
        let action = ConnectionRequestAction::from_code(bytes_to_int(action_bytes) as i32);
        let transaction_id = bytes_to_int(transaction_id_bytes) as i32;
        let connection_id = bytes_to_int(connection_id_bytes) as i64;
        Ok(
            ConnectionResponse {
                action: action?,
                transaction_id,
                connection_id,
            }
        )
    }
}

pub struct AnnounceRequest {
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
impl AnnounceRequest {
    pub(crate) fn new(connection_id: &i64, transaction_id: &i32, info_hash: Vec<u8>) -> Self {
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

    pub(crate) fn to_req_bytes(&self) -> Vec<u8> {
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

#[derive(Debug)]
pub struct AnnounceResponse {
    action: ConnectionRequestAction,
    transaction_id: i32,
    interval: i32,
    leechers: i32,
    seeders: i32,
    // IP address and TCP port
    pub(crate) peers: Vec<(String, i16)>,
}
impl AnnounceResponse {
    pub(crate) fn from_bytes(bytes: &[u8], len: usize) -> Result<Self, ()> {
        let action_bytes = &bytes[0..4];
        let transaction_id_bytes = &bytes[4..8];
        let interval_bytes = &bytes[8..12];
        let leechers_bytes = &bytes[12..16];
        let seeders_bytes = &bytes[16..20];
        let mut offset = 20;
        let mut peers = Vec::new();
        if offset + 6 <= len {
            while offset < len {
                let new_slice = sub_arr(bytes.to_vec(), offset, offset + 6);
                let ip_bytes = &new_slice[0..4];
                let port_bytes = &new_slice[4..6];
                let ip_string = format!("{}.{}.{}.{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
                peers.push((ip_string, bytes_to_int(port_bytes) as i16));
                offset += 6;
            }
        }
        Ok(
            AnnounceResponse {
                action: ConnectionRequestAction::from_code(bytes_to_int(action_bytes) as i32)?,
                transaction_id: bytes_to_int(transaction_id_bytes) as i32,
                interval: bytes_to_int(interval_bytes) as i32,
                leechers: bytes_to_int(leechers_bytes) as i32,
                seeders: bytes_to_int(seeders_bytes) as i32,
                peers,
            }
        )
    }
}
