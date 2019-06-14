mod tree_utils;

use std::collections::BTreeMap;
use tree_utils::Range;
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
            return;
        }

        let overlapping_ranges = tree_utils::ranges_within(
            &self.offset_tree,
            match start {
                0 => 0,
                other => other - 1,
            },
            end + 1,
        );

        // println!("Overlapping ranges! {:?}", overlapping_ranges);

        let mut first_pass_done: bool = false;
        let mut should_insert: bool = false;

        for Range {
            start: range_start,
            end: range_end,
            size: range_size,
        } in overlapping_ranges
        {
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
                if range_size != size {
                    // had an isNaN check here, we can probably use 0 for this special case
                    self.offset_tree.insert(end + 1, range_size);
                }
            }
        }

        if should_insert {
            self.offset_tree.insert(start, size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OffsetList;

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
    fn re_insert_at_start() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 5);
        list.insert(0, 0, 10);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(values, [10, 5]);
        assert_eq!(keys, [0, 1]);
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
    fn test_join_start() {
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
    fn test_join_end() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(5, 7, 20);
        list.insert(3, 5, 20);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(values, [10, 20, 10]);
        assert_eq!(keys, [0, 3, 8]);
    }

    #[test]
    fn test_override() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(5, 7, 20);
        list.insert(4, 7, 30);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(keys, [0, 4, 8]);
        assert_eq!(values, [10, 30, 10]);
    }

    #[test]
    fn test_join_override() {
        let mut list: OffsetList = OffsetList::new();

        list.insert(0, 0, 5);
        list.insert(4, 5, 10);
        list.insert(6, 7, 20);
        list.insert(3, 8, 5);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(keys, [0]);
        assert_eq!(values, [5]);
    }
}
