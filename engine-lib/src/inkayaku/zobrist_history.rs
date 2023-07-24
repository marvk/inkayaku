pub struct ZobristHistory {
    history: [u64; 5000],
}

impl ZobristHistory {
    pub fn set(&mut self, index: usize, zobrist_hash: u64) {
        self.history[index] = zobrist_hash;
    }

    pub fn is_threefold_repetition(&self, start_index: usize) -> bool {
        let mut current_index = (start_index as i32) - 4;
        let mut repetitions = 1;
        let zobrist = self.history[start_index];

        while current_index >= 0 {
            let current_zobrist = self.history[current_index as usize];
            if current_zobrist == zobrist {
                repetitions += 1;

                if repetitions >= 3 {
                    return true;
                }
            }

            current_index -= 2;
        }

        false
    }
}

impl Default for ZobristHistory {
    fn default() -> Self {
        ZobristHistory { history: [0; 5000] }
    }
}
