use std::collections::HashMap;
use std::ops::Index;
use crate::str_utils::{index_of, sub_arr, sub_str, vec_index_of};

type BString = Vec<u8>;
type BInt = i64;
type BDict = HashMap<String, Bencode>;
type BList = Vec<Bencode>;

#[derive(Debug, PartialEq)]
pub enum Bencode {
    Str(BString),
    Int(BInt),
    List(BList),
    Dict(BDict),
    End,
}

impl Bencode {
    fn new_str(str: impl Into<String>) -> Self {
        Bencode::Str(str.into().as_bytes().to_vec())
    }
}

enum BencodeTypes {
    Str,
    Int,
    List,
    Dict,
    End,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct ParseResult<T> {
    data: T,
    len: usize,
}
impl<T> ParseResult<T> {
    fn new(data: T, len: usize) -> Self
    {
        ParseResult {
            data,
            len,
        }
    }
}

pub fn parse_bencode(line: impl Into<String>) -> Result<ParseResult<Bencode>, ()>
{
    let line = line.into();
    let ben_type = get_type(&line);
    match ben_type {
        BencodeTypes::Str => {
            let res = parse_string(&line.as_bytes().to_vec());
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::Str(data), len))
            } else {
                panic!("Invalid String {}", line)
            }
        }
        BencodeTypes::Int => {
            let res = parse_int(&(line.as_bytes().to_vec()));
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::Int(data), len))
            } else {
                Err(())
            }
        }
        BencodeTypes::List => {
            let res = parse_list(&line);
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::List(data), len))
            } else {
                Err(())
            }
        }
        BencodeTypes::Dict => {
            let res = parse_dict(&line);
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::Dict(data), len))
            } else {
                Err(())
            }
        }
        BencodeTypes::End => {
            Ok(ParseResult::new(Bencode::End, 1))
        }
    }
}

fn get_type(line: impl Into<String>) -> BencodeTypes {
    let line = line.into();
    let first_char = sub_str(line, 0, 1);
    match first_char.as_str() {
        "i" => BencodeTypes::Int,
        "d" => BencodeTypes::Dict,
        "l" => BencodeTypes::List,
        "e" => BencodeTypes::End,
        "" => BencodeTypes::End,
        _ => BencodeTypes::Str,
    }
}
fn parse_string(line: &Vec<u8>) -> Result<ParseResult<BString>, ()> {
    // let line = line.into();
    // let separator_idx = index_of(&line, ':');
    let separator_idx = vec_index_of(line, ":".as_bytes()[0]);
    match separator_idx {
        Ok(separator_idx) => {
            let len = sub_str((String::from_utf8(line.clone()).unwrap_or_else(|_| { panic!("String length needs to be valid utf8") })), 0, separator_idx);
            let len = len.parse::<usize>().unwrap_or_else(|_| { panic!("Invalid string") });
            // let string = sub_str(&line, separator_idx + 1, len);
            let string = sub_arr(line.to_vec(), separator_idx + 1, len);
            Ok(ParseResult::new(string, separator_idx + 1 + len))
        }
        Err(_) => {
            Err(())
        }
    }
}
fn parse_int(line: &Vec<u8>) -> Result<ParseResult<BInt>, ()> {
    let first_char = sub_arr(line.to_vec(), 0, 1)[0];
    if first_char == "i".as_bytes()[0] {
        let index_of_end = vec_index_of(&line, "e".as_bytes()[0]);
        if let Ok(index_of_end) = index_of_end {
            let num = sub_str(&(String::from_utf8(line.to_vec()).unwrap_or_else(|_| { panic!("Integer needs to be valid utf8") })), 1, index_of_end - 1).parse().unwrap_or_else(|_| { panic!("Invalid Integer") });
            return Ok(ParseResult::new(num, index_of_end + 1));
        }
    }
    Err(())
}

fn parse_list(line: impl Into<String>) -> Result<ParseResult<BList>, ()> {
    let line = line.into();
    let first_char = sub_str(&line, 0, 1);
    if first_char == "l" {
        let mut new_line = sub_str(&line, 1, line.len());
        let mut ret_vec = Vec::new();
        let mut total_parsed = 1;
        loop {
            let bencode = parse_bencode(&new_line);
            if let Ok(res) = bencode {
                let ParseResult { data, len } = res;
                if Bencode::End == data {
                    total_parsed += len;
                    break;
                } else {
                    total_parsed += len;
                    new_line = sub_str(&new_line, len, new_line.len());
                    ret_vec.push(data)
                }
            }
        }
        return Ok(ParseResult::new(ret_vec, total_parsed));
    }
    Err(())
}
fn parse_dict(line: impl Into<String>) -> Result<ParseResult<BDict>, ()> {
    let line = line.into();
    let first_char = sub_str(&line, 0, 1);
    if first_char == "d" {
        let mut new_line = sub_str(&line, 1, line.len());
        let mut ret_map = HashMap::new();
        let mut total_parsed = 1;
        loop {
            let bencode_str = parse_bencode(&new_line);
            if let Ok(res) = bencode_str {
                let ParseResult { data, len } = res;
                total_parsed += len;
                if let Bencode::Str(map_key) = data {
                    new_line = sub_str(&new_line, len, new_line.len());
                    let benccode_value = parse_bencode(&new_line);
                    if let Ok(res) = benccode_value {
                        let ParseResult { data, len } = res;
                        total_parsed += len;
                        ret_map.insert(String::from_utf8(map_key).unwrap(), data);
                        new_line = sub_str(&new_line, len, new_line.len())
                    } else {
                        panic!("Invalid bencode")
                    }
                } else if data == Bencode::End {
                    break;
                } else {
                    panic!("Invalid bencode")
                }
            }
        }
        return Ok(ParseResult::new(ret_map, total_parsed));
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string(&"4:abcd".as_bytes().to_vec()), Ok(ParseResult::new("abcd".as_bytes().to_vec(), 6)));
        assert_eq!(parse_string(&"0:".as_bytes().to_vec()), Ok(ParseResult::new("".as_bytes().to_vec(), 2)))
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int(&"i123e".as_bytes().to_vec()), Ok(ParseResult::new(123, 5)))
    }

    #[test]
    fn test_list() {
        let test_str = String::from("l4:spam4:eggsi-234el4:spam4:eggsi-234e4:mdheee");
        let lhs = parse_list(&test_str);
        let rhs = Ok(
            ParseResult::new(
                vec![
                    Bencode::new_str("spam"),
                    Bencode::new_str("eggs"),
                    Bencode::Int(-234),
                    Bencode::List(vec![
                        Bencode::new_str("spam"),
                        Bencode::new_str("eggs"),
                        Bencode::Int(-234),
                        Bencode::new_str("mdhe")
                    ]
                    )
                ]
                , test_str.len()));
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_dict() {
        let test_str = String::from("d4:listli12e3:zln6:whatupd1:k1:vee4:mdhe4:here3:numi-234ee");
        let lhs = parse_dict(&test_str);

        let mut map = HashMap::new();
        let mut inner_map = HashMap::new();
        inner_map.insert(String::from("k"), Bencode::new_str("v"));
        map.insert(String::from("list"), Bencode::List(
            vec![
                Bencode::Int(12),
                Bencode::new_str("zln"),
                Bencode::new_str("whatup"),
                Bencode::Dict(inner_map)
            ]
        ));
        map.insert(String::from("mdhe"), Bencode::new_str("here"));
        map.insert(String::from("num"), Bencode::Int(-234));
        let rhs = Ok(ParseResult::new(map, test_str.len()));
        assert_eq!(lhs, rhs);
    }
}
