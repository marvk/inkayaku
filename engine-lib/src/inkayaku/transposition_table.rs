use std::collections::{HashMap, LinkedList};

use marvk_chess_board::board::constants::ZobristHash;
use crate::inkayaku::search::ValuedMove;

pub enum NodeType {
    Exact,
    Lowerbound,
    Upperbound,
}

pub struct TtEntry {
    pub mv: ValuedMove,
    pub zobrist_hash: ZobristHash,
    pub depth: usize,
    pub value: i32,
    pub node_type: NodeType,
}

impl TtEntry {
    pub fn new(mv: ValuedMove, zobrist_hash: ZobristHash, depth: usize, value: i32, node_type: NodeType) -> Self {
        Self { mv, zobrist_hash, depth, value, node_type }
    }
}

pub trait TranspositionTable {
    fn clear(&mut self);
    fn put(&mut self, zobrist_hash: ZobristHash, entry: TtEntry);
    fn get(&self, zobrist_hash: ZobristHash) -> Option<&TtEntry>;
    fn len(&self) -> usize;
    fn load_factor(&self) -> f32;
}

pub struct ArrayTranspositionTable<const N: usize> {
    entries: Vec<Option<TtEntry>>,
    load: usize,
}

impl<const N: usize> ArrayTranspositionTable<N> {
    pub fn new() -> Self {
        Self { entries: Self::new_vec(), load: 0 }
    }

    fn new_vec() -> Vec<Option<TtEntry>> {
        (0..N).map(|_| None).collect()
    }

    const fn array_hash(hash: u64) -> usize {
        (hash % N as u64) as usize
    }
}

impl<const N: usize> TranspositionTable for ArrayTranspositionTable<N> {
    fn clear(&mut self) {
        self.entries = Self::new_vec();
    }

    fn put(&mut self, zobrist_hash: ZobristHash, entry: TtEntry) {
        let hash = Self::array_hash(zobrist_hash);
        let option = &mut self.entries[hash];
        if option.is_none() {
            self.load+=1;
        }
        *option = Some(entry);
    }

    fn get(&self, zobrist_hash: ZobristHash) -> Option<&TtEntry> {
        let array_hash = Self::array_hash(zobrist_hash);
        if let Some(entry) = &self.entries[array_hash] {
            if entry.zobrist_hash == zobrist_hash {
                Some(entry)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.load
    }

    fn load_factor(&self) -> f32 {
        self.len() as f32 / N as f32
    }
}

pub struct HashMapTranspositionTable {
    capacity: usize,
    entry_list: LinkedList<u64>,
    entry_map: HashMap<u64, TtEntry>,
}

impl HashMapTranspositionTable {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, entry_list: LinkedList::new(), entry_map: HashMap::with_capacity(capacity) }
    }
}

impl TranspositionTable for HashMapTranspositionTable {
    fn clear(&mut self) {
        self.entry_list.clear();
        self.entry_map.clear();
    }

    fn put(&mut self, zobrist_hash: ZobristHash, entry: TtEntry) {
        if self.entry_map.insert(zobrist_hash, entry).is_none() {
            self.entry_list.push_back(zobrist_hash);
        }
        if self.entry_map.len() > self.capacity {
            let remove_key = self.entry_list.pop_front().unwrap();
            self.entry_map.remove(&remove_key);
        }
    }

    fn get(&self, zobrist_hash: ZobristHash) -> Option<&TtEntry> {
        self.entry_map.get(&zobrist_hash)
    }

    fn len(&self) -> usize {
        self.entry_map.len()
    }

    fn load_factor(&self) -> f32 {
        self.len() as f32 / self.capacity as f32
    }
}

#[cfg(test)]
mod test {
    use crate::inkayaku::search::ValuedMove;
    use crate::inkayaku::transposition_table::{HashMapTranspositionTable, NodeType, TranspositionTable, TtEntry};

    fn gen_value() -> TtEntry {
        TtEntry::new(ValuedMove::leaf(0), 0, 0, 0, NodeType::Exact)
    }

    #[test]
    fn clear_oldest() {
        let mut sut = HashMapTranspositionTable::new(3);

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

    fn assert_len(sut: &mut HashMapTranspositionTable, len: usize) {
        assert_eq!(sut.len(), len);
        assert_eq!(sut.entry_list.len(), len);
        assert_eq!(sut.entry_map.len(), len);
    }
}
