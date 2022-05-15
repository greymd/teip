use regex::Regex;

pub fn trim_eol(buf: &mut Vec<u8>) -> String {
    if buf.ends_with(&[b'\r', b'\n']) {
        buf.pop();
        buf.pop();
        return "\r\n".to_string();
    }
    if buf.ends_with(&[b'\n']) {
        buf.pop();
        return "\n".to_string();
    }
    if buf.ends_with(&[b'\0']) {
        buf.pop();
        return "\0".to_string();
    }
    "".to_string()
}

// Extract number from string line
pub fn extract_number(line: String) -> Option<u64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\s*([0-9]+)").unwrap();
    }
    let iter = RE.captures_iter(&line);
    for cap in iter {
        let s = &cap[1];
        let i: u64 = s.parse().unwrap();
        return Some(i)
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_trim_eol() {
        let mut buf = vec![b'\x61', b'\x62', b'\n'];
        let end = trim_eol(&mut buf);
        assert_eq!(String::from_utf8_lossy(&buf).to_string(), "ab");
        assert_eq!(end, "\n");
    }
    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("1234:abc".to_string()), Some(1234));
        assert_eq!(extract_number("0123:abc".to_string()), Some(123));
        assert_eq!(extract_number("     065535   :abc".to_string()), Some(65535));
        assert_eq!(extract_number("hoge fuga 123 5g".to_string()), None);
        assert_eq!(extract_number("\t999\t123".to_string()), Some(999));
        assert_eq!(extract_number("0".to_string()), Some(0));
        assert_eq!(extract_number("43201613413".to_string()), Some(43201613413));
        assert_eq!(extract_number("-1".to_string()), None);
    }
}
