use std::collections::HashMap;
use crate::str_utils::{index_of, sub_str};

#[derive(Debug)]
#[derive(PartialEq)]
enum BString {
    Str(String)
}
impl BString {
    fn new(string: impl Into<String>) -> Self
    {
        BString::Str(string.into())
    }
}


#[derive(Debug)]
enum BInt {
    Int(i64)
}

#[derive(Debug)]
enum BList {
    List(Vec<Bencode>)
}

#[derive(Debug)]
enum BDict {
    Dict(HashMap<BString, Bencode>)
}

#[derive(Debug)]
enum Bencode {
    Str(BString),
    Int(BInt),
    List(BList),
    Dict(BDict),
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

pub fn parse_bencode(line: impl Into<String>) {
    let line = line.into();
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
fn parse_int(line_writer:impl Into<String>) -> Result<ParseResult<BInt>,()>{
    
    Err(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string("4:abcd"), Ok(ParseResult::new(BString::new("abcd"), 6)));
    }
}
