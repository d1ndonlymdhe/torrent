use std::collections::HashMap;
use crate::str_utils::{index_of, sub_str};

#[derive(Debug, PartialEq)]
enum BString {
    Str(String)
}
impl BString {
    fn new(string: impl Into<String>) -> Self
    {
        BString::Str(string.into())
    }
}


#[derive(Debug, PartialEq)]
enum BInt {
    Int(i64)
}

#[derive(Debug, PartialEq)]
enum BList {
    List(Vec<Bencode>)
}

#[derive(Debug)]
enum BDict {
    Dict(HashMap<String, Bencode>)
}
impl PartialEq for BDict {
    fn eq(&self, other: &Self) -> bool {
        let Self::Dict(map) = self;
        let Self::Dict(other_map) = other;
        let keys: Vec<&String> = map.keys().collect();
        for key in keys {
            let try_get_other = other_map.get(key);
            if let Some(other_value) = try_get_other {
                let self_value = map.get(key).unwrap();
                if self_value != other_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, PartialEq)]
enum Bencode {
    Str(BString),
    Int(BInt),
    List(BList),
    Dict(BDict),
    End,
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
struct ParseResult<T> {
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
            let res = parse_string(&line);
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::Str(data), len))
            } else {
                Err(())
            }
        }
        BencodeTypes::Int => {
            let res = parse_int(&line);
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
        _ => BencodeTypes::Str,
    }
}
fn parse_string(line: impl Into<String>) -> Result<ParseResult<BString>, ()> {
    let line = line.into();
    let separator_idx = index_of(&line, ':');
    match separator_idx {
        Ok(separator_idx) => {
            let len = sub_str(&line, 0, separator_idx);
            let len = len.parse::<usize>().unwrap_or_else(|_| { panic!("Invalid string") });
            let string = sub_str(&line, separator_idx + 1, len);
            Ok(ParseResult::new(BString::Str(string), separator_idx + 1 + len))
        }
        Err(_) => {
            Err(())
        }
    }
}
fn parse_int(line: impl Into<String>) -> Result<ParseResult<BInt>, ()> {
    let line = line.into();
    let first_char = sub_str(&line, 0, 1);
    if first_char == "i" {
        let index_of_end = index_of(&line, 'e');
        if let Ok(index_of_end) = index_of_end {
            let num = sub_str(&line, 1, index_of_end - 1).parse().unwrap_or_else(|_| { panic!("Invalid Integer") });
            return Ok(ParseResult::new(BInt::Int(num), index_of_end + 1));
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
        return Ok(ParseResult::new(BList::List(ret_vec), total_parsed));
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
                        let BString::Str(map_key) = map_key;
                        total_parsed += len;
                        ret_map.insert(map_key, data);
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
        return Ok(ParseResult::new(BDict::Dict(ret_map), total_parsed));
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string("4:abcd"), Ok(ParseResult::new(BString::new("abcd"), 6)));
        assert_eq!(parse_string("0:"), Ok(ParseResult::new(BString::new(""), 2)))
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("i123e"), Ok(ParseResult::new(BInt::Int(123), 5)))
    }

    #[test]
    fn test_list() {
        let test_str = String::from("l4:spam4:eggsi-234el4:spam4:eggsi-234e4:mdheee");
        let lhs = parse_list(&test_str);
        let rhs = Ok(
            ParseResult::new(
                BList::List(vec![
                    Bencode::Str(BString::new("spam")),
                    Bencode::Str(BString::new("eggs")),
                    Bencode::Int(BInt::Int(-234)),
                    Bencode::List(BList::List(vec![
                        Bencode::Str(BString::new("spam")),
                        Bencode::Str(BString::new("eggs")),
                        Bencode::Int(BInt::Int(-234)),
                        Bencode::Str(BString::new("mdhe"))
                    ]
                    ))
                ]
                ), test_str.len()));
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_dict() {
        let test_str = String::from("d4:listli12e3:zln6:whatupd1:k1:vee4:mdhe4:here3:numi-234ee");
        let lhs = parse_dict(&test_str);

        let mut map = HashMap::new();
        let inner_map = HashMap::new();
        map.insert(String::from("k"), Bencode::Str(BString::new("V")));
        map.insert(String::from("list"), Bencode::List(BList::List(
            vec![
                Bencode::Int(BInt::Int(12)),
                Bencode::Str(BString::new("zln")),
                Bencode::Str(BString::new("whatup")),
                Bencode::Dict(BDict::Dict(inner_map))
            ]
        )));
        map.insert(String::from("mdhe"), Bencode::Str(BString::new("here")));
        map.insert(String::from("num"), Bencode::Int(BInt::Int(-234)));


        let rhs = Ok(ParseResult::new(BDict::Dict(map), test_str.len()));
        assert_eq!(lhs, rhs);
    }
}
