pub fn bytes_to_int(bytes: &[u8]) -> i128 {
    let mut num = 0;
    for (idx, byte) in bytes.iter().rev().enumerate() {
        let byte = *byte as i128;
        num += byte * 0x100i128.pow(idx as u32);
    }
    num
}

pub fn parse_url(url: impl Into<String>) -> (String, String, String) {
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

pub fn int_to_bytes(int: i128, size: usize) -> Vec<u8> {
    let mut int = int;
    let mut bytes = Vec::new();
    for _ in 0..size {
        let r = (int % 0x100) as u8;
        int /= 0x100;
        bytes.push(r);
    }
    bytes
}
