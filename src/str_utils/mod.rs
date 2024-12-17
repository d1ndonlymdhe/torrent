pub fn sub_str(string: impl Into<String>, start: usize, len: usize) -> String {
    string.into().chars().skip(start).take(len).collect()
}
pub fn sub_arr<T>(vec: Vec<T>, start: usize, len: usize) -> Vec<T> {
    vec.into_iter().skip(start).take(len).collect()
}
pub fn vec_index_of<T: PartialEq>(vec: &Vec<T>, item: T) -> Result<usize, ()> {
    if let Some(d) = vec.iter().enumerate().find(|x| { *x.1 == item }) {
        Ok(d.0)
    } else {
        Err(())
    }
}
pub fn index_of(string: impl Into<String>, search_char: char) -> Result<usize, ()> {
    if let Some(d) = string.into().chars().enumerate().find(|x| { x.1 == search_char }) {
        Ok(d.0)
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sub_str() {
        assert_eq!(sub_str("HELLO", 1, 3), String::from("ELL"));
    }
    #[test]
    fn test_index() {
        assert_eq!(index_of("HELLO", 'E'), Ok(1));
        assert_eq!(index_of("HELLO", 'L'), Ok(2));
        assert_eq!(index_of("HELLO", 'Z'), Err(()));
    }
}