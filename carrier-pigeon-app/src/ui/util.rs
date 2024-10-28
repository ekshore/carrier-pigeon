pub fn convert_case(str: String) -> String {
    let bytes = str.as_bytes().to_owned();
    let mut return_bytes: Vec<u8> = Vec::with_capacity(bytes.len());
    return_bytes.push(bytes[0]);

    for byte in bytes.iter().skip(1) {
        if *byte < b'a' {
            return_bytes.push(b' ');
        }
        return_bytes.push(*byte);
    }
    String::from_utf8(return_bytes).expect("If the blows up we have bigger problems")
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn convert_case_conversion() {
        let converted_string = convert_case("HelloWorld".into());
        assert_eq!("Hello World", converted_string);
    }

    #[test]
    fn convert_case_no_conversion() {
        let converted_string = convert_case("Hello".into());
        assert_eq!("Hello", converted_string);
    }
}
