use std::collections::BTreeMap;

use std::ops::Bound::Included;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OffsetList {
    offset_tree: BTreeMap<u32, u32>,
}

#[wasm_bindgen]
impl OffsetList {
    pub fn new() -> OffsetList {
        OffsetList {
            offset_tree: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, start: u32, end: u32, size: u32) {
        if self.offset_tree.is_empty() {
            self.offset_tree.insert(0, size);
            return
        }

        let overlapping_ranges = ranges_within(&self.offset_tree, start - 1, end + 1);

        let mut first_pass_done: bool = false;
        let mut should_insert: bool = false;

        for Range { start: range_start, end: range_end, size: range_size } in overlapping_ranges {
            // previous range
            if !first_pass_done {
                should_insert = range_size != size;
                first_pass_done = true;
            } else {
                // remove the range if it starts within the new range OR if
                // it has the same value as it, in order to perfrom a merge
                if end >= range_start || size == range_size {
                    self.offset_tree.remove(&range_start);
                }
            }

            // next range
            if range_end > end && end >= range_start {
                if range_size != size { // had an isNaN check here, we can probably use 0 for this special case
                    self.offset_tree.insert(end + 1, range_size);
                }
            }
        }

        if should_insert {
            self.offset_tree.insert(start, size);
        }

    }
}

#[derive(Debug)]
struct Range {
    start: u32,
    end: u32,
    size: u32,
}

fn ranges_within(tree: &BTreeMap<u32, u32>, start: u32, end: u32) -> Vec<Range> {
    let mut ranges: Vec<Range> = Vec::new();

    let (start_key, _) = tree
        .range((Included(&0), Included(&start)))
        .last()
        .expect("Tree should contain zero");

    let mut key_vals = tree.range(start_key..&end);

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
    use super::OffsetList;
    use std::collections::BTreeMap;

    #[test]
    fn test_initial_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(values, [10]);
        assert_eq!(keys, [0]);
    }

    #[test]
    fn test_same_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(1, 1, 10);
        list.insert(20, 21, 10);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(values, [10]);
        assert_eq!(keys, [0]);
    }

    #[test]
    fn test_new_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(3, 5, 20);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(values, [10, 20, 10]);
        assert_eq!(keys, [0, 3, 6]);
    }

    #[test]
    fn test_join() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(3, 5, 20);
        list.insert(5, 7, 20);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(values, [10, 20, 10]);
        assert_eq!(keys, [0, 3, 8]);
    }

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
