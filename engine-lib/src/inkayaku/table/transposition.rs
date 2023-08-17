use marvk_chess_board::constants::ZobristHash;

use crate::inkayaku::search::ValuedMove;
use crate::inkayaku::table::HashTable;

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
    pub const fn new(mv: ValuedMove, zobrist_hash: ZobristHash, depth: usize, value: i32, node_type: NodeType) -> Self {
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
    fn new_vec() -> Vec<Option<TtEntry>> {
        (0..N).map(|_| None).collect()
    }

    const fn array_hash(hash: u64) -> usize {
        (hash % N as u64) as usize
    }
}

impl<const N: usize> Default for ArrayTranspositionTable<N> {
    fn default() -> Self {
        Self { entries: Self::new_vec(), load: 0 }
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
            self.load += 1;
        }
        *option = Some(entry);
    }

    fn get(&self, zobrist_hash: ZobristHash) -> Option<&TtEntry> {
        let array_hash = Self::array_hash(zobrist_hash);
        self.entries[array_hash].as_ref().filter(|entry| entry.zobrist_hash == zobrist_hash)
    }

    fn len(&self) -> usize {
        self.load
    }

    fn load_factor(&self) -> f32 {
        self.len() as f32 / N as f32
    }
}

pub struct HashMapTranspositionTable {
    hash_table: HashTable<ZobristHash, TtEntry>,
}

impl HashMapTranspositionTable {
    pub fn new(capacity: usize) -> Self {
        Self { hash_table: HashTable::new(capacity) }
    }
}

impl TranspositionTable for HashMapTranspositionTable {
    fn clear(&mut self) {
        self.hash_table.clear();
    }

    fn put(&mut self, zobrist_hash: ZobristHash, entry: TtEntry) {
        self.hash_table.put(zobrist_hash, entry);
    }

    fn get(&self, zobrist_hash: ZobristHash) -> Option<&TtEntry> {
        self.hash_table.get(zobrist_hash)
    }

    fn len(&self) -> usize {
        self.hash_table.len()
    }

    fn load_factor(&self) -> f32 {
        self.hash_table.load_factor()
    }
}
