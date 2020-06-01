use super::ranges::{self, Range};

pub fn to_ranges(list: &str, complement: bool) -> Result<Vec<Range>, String> {
    if complement {
        Range::from_list(list).map(|r| ranges::complement(&r))
    } else {
        Range::from_list(list)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_to_ranges() {
        let range = to_ranges("2-5,1-8", false).unwrap();
        assert_eq!(range[0].low, 1);
        assert_eq!(range[0].high, 8);
    }
}
