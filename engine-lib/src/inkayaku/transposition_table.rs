use std::collections::{HashMap, LinkedList};

use crate::inkayaku::ValuedMove;

pub enum NodeType {
    EXACT,
    LOWERBOUND,
    UPPERBOUND,
}

pub struct TtEntry {
    pub mv: ValuedMove,
    pub depth: usize,
    pub value: i32,
    pub node_type: NodeType,
}

impl TtEntry {
    pub fn new(mv: ValuedMove, depth: usize, value: i32, node_type: NodeType) -> Self {
        Self { mv, depth, value, node_type }
    }
}

pub struct TranspositionTable {
    capacity: usize,
    entry_list: LinkedList<u64>,
    entry_map: HashMap<u64, TtEntry>,
}


impl TranspositionTable {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, entry_list: LinkedList::new(), entry_map: HashMap::with_capacity(capacity) }
    }

    pub fn clear(&mut self) {
        self.entry_list.clear();
        self.entry_map.clear();
    }

    pub fn put(&mut self, hash: u64, entry: TtEntry) {
        if self.entry_map.insert(hash, entry).is_none() {
            self.entry_list.push_back(hash);
        }
        if self.entry_map.len() > self.capacity {
            let remove_key = self.entry_list.pop_front().unwrap();
            self.entry_map.remove(&remove_key);
        }
    }

    pub fn get(&self, hash: u64) -> Option<&TtEntry> {
        self.entry_map.get(&hash)
    }

    pub fn len(&self) -> usize {
        self.entry_map.len()
    }

    pub fn load_factor(&self) -> f64 {
        self.len() as f64 / self.capacity as f64
    }
}

#[cfg(test)]
mod test {
    use crate::inkayaku::transposition_table::{NodeType, TranspositionTable, TtEntry};
    use crate::inkayaku::ValuedMove;

    fn gen_value() -> TtEntry {
        TtEntry::new(ValuedMove::leaf(0), 0, 0, NodeType::EXACT)
    }

    #[test]
    fn clear_oldest() {
        let mut sut = TranspositionTable::new(3);

        sut.put(1, gen_value());
        assert_len(&mut sut, 1);
        sut.put(1, gen_value());
        assert_len(&mut sut, 1);
        sut.put(2, gen_value());
        assert_len(&mut sut, 2);
        sut.put(2, gen_value());
        assert_len(&mut sut, 2);
        sut.put(3, gen_value());
        assert_len(&mut sut, 3);
        sut.put(4, gen_value());
        assert_len(&mut sut, 3);
        sut.put(1, gen_value());
        assert_len(&mut sut, 3);
    }

    fn assert_len(sut: &mut TranspositionTable, len: usize) {
        assert_eq!(sut.len(), len);
        assert_eq!(sut.entry_list.len(), len);
        assert_eq!(sut.entry_map.len(), len);
    }
}
