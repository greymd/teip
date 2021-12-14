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

pub fn vecstr_rm_references(orig: &Vec<&str>) -> Vec<String> {
    let mut removed: Vec<String> = Vec::new();
    for c in orig {
        removed.push(c.to_string());
    }
    removed
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
}
