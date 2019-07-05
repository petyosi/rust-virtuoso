mod tree_utils;

use std::cmp;
use std::collections::BTreeMap;
use tree_utils::Range;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Item {
    size: u32,
    offset: u32,
    index: u32,
}

#[wasm_bindgen]
pub struct OffsetList {
    size_tree: BTreeMap<u32, u32>,
    offset_tree: BTreeMap<u32, u32>,
    pixel_tree: BTreeMap<u32, u32>,
}

#[wasm_bindgen]
impl OffsetList {
    pub fn new() -> OffsetList {
        OffsetList {
            size_tree: BTreeMap::new(),
            offset_tree: BTreeMap::new(),
            pixel_tree: BTreeMap::new(),
        }
    }

    pub fn update_offset_tree(&mut self, start: u32) {
        let lte = match start {
            0 => 0,
            other => other - 1,
        };

        let updated = self.size_tree.range(lte..);

        let (start_index, start_size) = tree_utils::lte(&self.size_tree, lte);

        let mut prev_offset = match self.offset_tree.get(start_index) {
            None => 0u32,
            Some(offset) => *offset,
        };

        let mut prev_size = start_size;
        let mut prev_index = start_index;
        for (index, size) in updated {
            let offset = (index - prev_index) * prev_size + prev_offset;
            self.offset_tree.insert(*index, offset);
            self.pixel_tree.insert(offset, *index);
            prev_index = index;
            prev_offset = offset;
            prev_size = size;
        }
    }

    pub fn remove_index(&mut self, index: &u32) {
        self.size_tree.remove(index);
        let pixel = self
            .offset_tree
            .remove(index)
            .expect("offset tree should be in sync!");

        self.pixel_tree.remove(&pixel);
    }

    pub fn insert_spots(&mut self, spots: Vec<u32>, size: u32) {
        if !self.size_tree.is_empty() {
            panic!("Trying to insert spots in non-empty size tree.");
        }

        for spot in spots.iter() {
            self.size_tree.insert(*spot, size);
            self.size_tree.insert(spot + 1, 0);
        }

        self.update_offset_tree(0);
    }

    pub fn insert(&mut self, start: u32, end: u32, size: u32) {
        if self.size_tree.is_empty() {
            self.size_tree.insert(0, size);
            self.update_offset_tree(start);
            return;
        }

        if let Some(0) = self.size_tree.get(&start) {
            let group_size = self
                .size_tree
                .get(&(start - 1))
                .expect("We must have a group size if zero sized element is present");

            if *group_size == size {
                self.size_tree = BTreeMap::new();
                self.size_tree.insert(0, size);
                self.offset_tree = BTreeMap::new();
                self.offset_tree.insert(0, 0);
                return;
            } else {
                for (_key, value) in self.size_tree.iter_mut() {
                    if value == &0 {
                        *value = size;
                    }
                }
                self.update_offset_tree(start);
                return;
            }
        }

        let overlapping_ranges = tree_utils::ranges_within(
            &self.size_tree,
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
                    self.remove_index(&range_start);
                }
            }

            // next range
            if range_end > end && end >= range_start {
                if range_size != size {
                    // had an isNaN check here, we can probably use 0 for this special case
                    self.size_tree.insert(end + 1, range_size);
                }
            }
        }

        if should_insert {
            self.size_tree.insert(start, size);
        }

        self.update_offset_tree(start);
    }

    pub fn offset_of(self, index: u32) -> u32 {
        let (size, offset, range_index) = self.range_size_and_offset(index);

        return (index - range_index) * size + offset;
    }

    pub fn total(self, index: u32) -> u32 {
        let (size, offset, range_index) = self.range_size_and_offset(index);

        return (index - range_index + 1) * size + offset;
    }

    pub fn item_at(self, index: u32) -> Item {
        let (size, offset, range_index) = self.range_size_and_offset(index);
        return Item {
            index: index,
            size: *size,
            offset: (index - range_index) * size + offset,
        };
    }

    pub fn index_range(&self, start_index: u32, end_index: u32) -> Vec<Item> {
        if self.size_tree.is_empty() {
            return vec![Item {
                index: 0,
                size: 0,
                offset: 0,
            }];
        }

        let ranges = tree_utils::ranges_within(&self.size_tree, start_index, end_index);
        let mut result: Vec<Item> = Vec::new();

        for range in ranges {
            let start = std::cmp::max(start_index, range.start);
            let end = std::cmp::min(range.end, end_index);

            for index in start..=end {
                result.push(Item {
                    index,
                    size: range.size,
                    offset: 0,
                })
            }
        }

        return result;
    }

    pub fn range(
        &self,
        start_offset: u32,
        end_offset: u32,
        min_index: u32,
        max_index: u32,
    ) -> Vec<Item> {
        let (_, start_index) = tree_utils::lte(&self.pixel_tree, start_offset);

        let (_, end_index) = self
            .pixel_tree
            .range(end_offset..)
            .next()
            .expect("we should find such end index");

        let mut result: Vec<Item> = Vec::new();

        for range in tree_utils::ranges_within(&self.offset_tree, *start_index, *end_index) {
            let mut offset = range.size;
            let mut start_index = range.start;
            let size = self
                .size_tree
                .get(&start_index)
                .expect("tree should be in sync");

            if range.size < start_offset {
                start_index += (start_offset - range.size) as u32 / size;
                offset += (start_index - range.start) * size;
            }

            if start_index < min_index {
                offset += (min_index - start_index) * size;
                start_index = min_index;
            }

            if size == &0u32 {
                result.push(Item {
                    index: start_index,
                    size: 0,
                    offset,
                });

                return result;
            }

            let end_index = cmp::min(range.end, max_index);

            for index in start_index..=end_index {
                if offset > end_offset {
                    break;
                }

                result.push(Item {
                    index: index,
                    size: *size,
                    offset: offset,
                });

                offset += size;
            }
        }

        return result;
    }

    fn range_size_and_offset(&self, index: u32) -> (&u32, &u32, &u32) {
        let (range_index, _) = tree_utils::lte(&self.size_tree, index);
        let size = self
            .size_tree
            .get(range_index)
            .expect("size tree should include the found index");
        let offset = self
            .offset_tree
            .get(range_index)
            .expect("offset tree should mirror the size tree");

        return (&size, &offset, &range_index);
    }
}

#[cfg(test)]
mod tests {
    use super::Item;
    use super::OffsetList;
    #[test]
    fn test_initial_offset_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(values, [0]);
        assert_eq!(keys, [0]);
    }

    #[test]
    fn test_second_offset_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(3, 7, 20);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(keys, [0, 3, 8]);
        assert_eq!(values, [0, 30, 130]);
    }

    #[test]
    fn test_in_between_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 1);
        list.insert(9, 10, 2);
        list.insert(3, 7, 3);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(keys, [0, 3, 8, 9, 11]);
        assert_eq!(values, [0, 3, 18, 19, 23]);
    }

    #[test]
    fn test_overlap_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 1);
        list.insert(3, 7, 2);
        list.insert(2, 9, 3);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(keys, [0, 2, 10]);
        assert_eq!(values, [0, 2, 26]);
    }

    #[test]
    fn test_initial_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(values, [10]);
        assert_eq!(keys, [0]);
    }

    #[test]
    fn test_same_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(1, 1, 10);
        list.insert(20, 21, 10);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(values, [10]);
        assert_eq!(keys, [0]);
    }

    #[test]
    fn re_insert_at_start() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 5);
        list.insert(0, 0, 10);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(values, [10, 5]);
        assert_eq!(keys, [0, 1]);
    }

    #[test]
    fn test_new_insert() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(3, 5, 20);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(values, [10, 20, 10]);
        assert_eq!(keys, [0, 3, 6]);
    }

    #[test]
    fn test_join_start() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(3, 5, 20);
        list.insert(5, 7, 20);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(values, [10, 20, 10]);
        assert_eq!(keys, [0, 3, 8]);
    }

    #[test]
    fn test_join_end() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(5, 7, 20);
        list.insert(3, 5, 20);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(values, [10, 20, 10]);
        assert_eq!(keys, [0, 3, 8]);
    }

    #[test]
    fn test_override() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(5, 7, 20);
        list.insert(4, 7, 30);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
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

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(keys, [0]);
        assert_eq!(values, [5]);
    }

    #[test]
    fn test_insert_sports() {
        let mut list: OffsetList = OffsetList::new();

        list.insert_spots(vec![0, 10, 20], 5);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(keys, [0, 1, 10, 11, 20, 21]);
        assert_eq!(values, [5, 0, 5, 0, 5, 0]);

        let values: Vec<u32> = list.offset_tree.values().cloned().collect();
        let keys: Vec<u32> = list.offset_tree.keys().cloned().collect();
        assert_eq!(keys, [0, 1, 10, 11, 20, 21]);
        assert_eq!(values, [0, 5, 5, 10, 10, 15]);
    }

    #[test]
    fn test_insert_size_after_spot() {
        let mut list: OffsetList = OffsetList::new();

        list.insert_spots(vec![0, 10, 20], 5);
        list.insert(1, 5, 10);

        let values: Vec<u32> = list.size_tree.values().cloned().collect();
        let keys: Vec<u32> = list.size_tree.keys().cloned().collect();
        assert_eq!(keys, [0, 1, 10, 11, 20, 21]);
        assert_eq!(values, [5, 10, 5, 10, 5, 10]);
    }

    #[test]
    fn test_offset_of() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 1);
        list.insert(2, 4, 2);

        assert_eq!(list.offset_of(7), 10);
    }

    #[test]
    fn test_total() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 1);
        list.insert(2, 4, 2);

        assert_eq!(list.total(7), 11);
    }

    #[test]
    fn test_item_at() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 1);
        list.insert(2, 4, 2);

        let item = list.item_at(10);
        assert_eq!(item.size, 1);
        assert_eq!(item.offset, 13);
        assert_eq!(item.index, 10);
    }

    #[test]
    fn test_index_range() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 1);
        list.insert(2, 4, 2);

        let items: Vec<Item> = list.index_range(3, 6);
        assert_eq!(items.len(), 4);

        let item = items.get(0).expect("Item 0 is here");
        assert_eq!(item.index, 3);
        assert_eq!(item.size, 2);

        let item = items.get(3).expect("Item 0 is here");
        assert_eq!(item.index, 6);
        assert_eq!(item.size, 1);
    }

    #[test]
    fn test_range() {
        let mut list: OffsetList = OffsetList::new();
        list.insert(0, 0, 10);
        list.insert(2, 4, 20);

        let items: Vec<Item> = list.range(13, 79, 0, std::u32::MAX);
        assert_eq!(items.len(), 4);

        let item = items.get(0).expect("Item is here");
        assert_eq!(item.index, 1);
        assert_eq!(item.size, 10);
        assert_eq!(item.offset, 10);

        let item = items.get(1).expect("Item is here");
        assert_eq!(item.index, 2);
        assert_eq!(item.size, 20);
        assert_eq!(item.offset, 20);

        let item = items.get(2).expect("Item is here");
        assert_eq!(item.index, 3);
        assert_eq!(item.size, 20);
        assert_eq!(item.offset, 40);

        let item = items.get(3).expect("Item is here");
        assert_eq!(item.index, 4);
        assert_eq!(item.size, 20);
        assert_eq!(item.offset, 60);
    }
}
