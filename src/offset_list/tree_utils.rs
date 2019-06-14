use std::collections::BTreeMap;
use std::ops::Bound::Included;

#[derive(Debug)]
pub struct Range {
    pub start: u32,
    pub end: u32,
    pub size: u32,
}

pub fn ranges_within(tree: &BTreeMap<u32, u32>, start: u32, end: u32) -> Vec<Range> {
    let mut ranges: Vec<Range> = Vec::new();

    let (start_key, _) = tree
        .range((Included(&0), Included(&start)))
        .last()
        .expect("Tree should contain zero");

    let mut key_vals = tree.range((Included(start_key), Included(&end)));

    let (mut start, mut size) = key_vals.next().expect("We should have at least one match!");

    for (next_start, next_size) in key_vals {
        ranges.push(Range {
            start: *start,
            end: next_start - 1,
            size: *size,
        });
        size = next_size;
        start = next_start;
    }

    ranges.push(Range {
        start: *start,
        end: std::u32::MAX,
        size: *size,
    });

    return ranges;
}

#[cfg(test)]
mod tests {
    use super::ranges_within;
    use std::collections::BTreeMap;

    #[test]
    fn test_ranges_within() {
        let mut tree: BTreeMap<u32, u32> = BTreeMap::new();
        tree.insert(0, 10);

        let ranges = ranges_within(&tree, 5, 20);

        assert_eq!(ranges[0].size, 10);
        assert_eq!(ranges[0].start, 0);
        assert_eq!(ranges[0].end, std::u32::MAX);
    }

    #[test]
    fn test_ranges_within2() {
        let mut tree: BTreeMap<u32, u32> = BTreeMap::new();
        tree.insert(0, 10);
        tree.insert(5, 20);
        tree.insert(10, 8);
        tree.insert(20, 30);

        let ranges = ranges_within(&tree, 6, 27);

        assert_eq!(ranges.len(), 3);

        assert_eq!(ranges[0].start, 5);
        assert_eq!(ranges[1].start, 10);
        assert_eq!(ranges[2].start, 20);
        assert_eq!(ranges[2].end, std::u32::MAX);

        let ranges2 = ranges_within(&tree, 5, 18);
        assert_eq!(ranges2.len(), 2);
    }
}
