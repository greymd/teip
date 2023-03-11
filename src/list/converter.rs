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
    use ranges::RangeJoin::{Merge, Normal, Split};
    #[test]
    fn test_to_ranges() {
        let range = to_ranges("2-5,1-8", false).unwrap();
        assert_eq!(range[0].low, 1);
        assert_eq!(range[0].high, 8);
    }
    #[test]
    fn test_to_ranges_merge_unmerge() {
        let range = to_ranges("1-3,4~6", false).unwrap();
        assert_eq!(range[0].low, 1);
        assert_eq!(range[0].high, 3);
        assert_eq!(range[0].join, Normal);
        assert_eq!(range[1].low, 4);
        assert_eq!(range[1].high, 6);
        assert_eq!(range[1].join, Merge);
    }
    #[test]
    fn test_to_ranges_unsort() {
        let range = to_ranges("4,1-3,5~7", false).unwrap();
        println!("{:?}", range);
        assert_eq!(range[0].low, 1);
        assert_eq!(range[0].high, 3);
        assert_eq!(range[0].join, Normal);
        assert_eq!(range[1].low, 4);
        assert_eq!(range[1].high, 4);
        assert_eq!(range[1].join, Normal);
        assert_eq!(range[2].low, 5);
        assert_eq!(range[2].high, 7);
        assert_eq!(range[2].join, Merge);
    }
    #[test]
    fn test_to_ranges_split() {
        let range = to_ranges("1,2,3,4", false).unwrap();
        println!("{:?}", range);
        for i in 0..4 {
            assert_eq!(range[i].low, i + 1);
            assert_eq!(range[i].high, i + 1);
            assert_eq!(range[i].join, Normal);
        }
    }
    #[test]
    fn test_to_ranges_split_unsort() {
        let range = to_ranges("5,3,4,1,2", false).unwrap();
        println!("{:?}", range);
        for i in 0..5 {
            assert_eq!(range[i].low, i + 1);
            assert_eq!(range[i].high, i + 1);
            assert_eq!(range[i].join, Normal);
        }
    }
    #[test]
    fn test_to_ranges_overwrap() {
        let range = to_ranges("1-3,2~5", false).unwrap();
        println!("{:?}", range);
        assert_eq!(range[0].low, 1);
        assert_eq!(range[0].high, 5);
        assert_eq!(range[0].join, Normal);
    }
    #[test]
    fn test_to_ranges_three_different_range() {
        let range = to_ranges("1-3,5:10,12,13~15", false).unwrap();
        println!("{:?}", range);
        assert_eq!(range[0].low, 1);
        assert_eq!(range[0].high, 3);
        assert_eq!(range[0].join, Normal);
        assert_eq!(range[1].low, 5);
        assert_eq!(range[1].high, 10);
        assert_eq!(range[1].join, Split);
        assert_eq!(range[2].low, 12);
        assert_eq!(range[2].high, 12);
        assert_eq!(range[2].join, Normal);
        assert_eq!(range[3].low, 13);
        assert_eq!(range[3].high, 15);
        assert_eq!(range[3].join, Merge);
    }
}
