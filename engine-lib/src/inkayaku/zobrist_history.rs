pub struct ZobristHistory {
    history: [u64; 5000],
}

impl ZobristHistory {
    pub fn set(&mut self, index: usize, zobrist_hash: u64) {
        self.history[index] = zobrist_hash;
    }
}

impl Default for ZobristHistory {
    fn default() -> Self {
        ZobristHistory { history: [0; 5000] }
    }
}
