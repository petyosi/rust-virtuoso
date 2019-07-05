use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Range {
    pub start: u32,
    pub end: u32,
    pub size: u32,
}

impl Range {
    fn new(start: u32, end: u32, size: u32) -> Self {
        Range { start, end, size }
    }
}

pub const LAST_RANGE_END: u32 = std::u32::MAX;

pub fn lte(tree: &BTreeMap<u32, u32>, start: u32) -> (&u32, &u32) {
    tree.range(..=start)
        .last()
        .expect("Tree should contain zero")
}

pub fn ranges_within(tree: &BTreeMap<u32, u32>, start: u32, end: u32) -> Vec<Range> {
    let mut ranges: Vec<Range> = Vec::new();

    let (closest_lte, _) = lte(tree, start);

    let mut nodes = tree.range(closest_lte..=&end);

    let (mut start, mut size) = nodes.next().expect("We should have at least one match!");

    for (next_start, next_size) in nodes {
        ranges.push(Range::new(*start, next_start - 1, *size));
        size = next_size;
        start = next_start;
    }

    ranges.push(Range::new(*start, LAST_RANGE_END, *size));

    return ranges;
}

#[cfg(test)]
mod tests {
    use super::ranges_within;
    use super::Range;
    use super::LAST_RANGE_END;
    use std::cmp::PartialEq;
    use std::collections::BTreeMap;

    impl PartialEq for Range {
        fn eq(&self, other: &Self) -> bool {
            self.start == other.start && self.end == other.end && self.size == other.size
        }
    }

    #[test]
    fn test_ranges_within() {
        let mut tree: BTreeMap<u32, u32> = BTreeMap::new();
        tree.insert(0, 10);

        let ranges = ranges_within(&tree, 5, 20);

        assert_eq!(ranges[..], [Range::new(0, LAST_RANGE_END, 10)]);
    }

    #[test]
    fn test_ranges_within2() {
        let mut tree: BTreeMap<u32, u32> = BTreeMap::new();
        tree.insert(0, 10);
        tree.insert(5, 20);
        tree.insert(10, 8);
        tree.insert(20, 30);

        let ranges = ranges_within(&tree, 6, 27);

        assert_eq!(
            ranges[..],
            [
                Range::new(5, 9, 20),
                Range::new(10, 19, 8),
                Range::new(20, LAST_RANGE_END, 30),
            ]
        )
    }
}
