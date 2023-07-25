use std::cmp::max;
use std::ops::Sub;

pub struct ZobristHistory {
    history: [u64; 5000],
}

impl ZobristHistory {
    pub fn set(&mut self, index: u32, zobrist_hash: u64) {
        self.history[index as usize] = zobrist_hash;
    }

    pub fn is_threefold_repetition(&self, start_index: u32, halfmove_clock: u32) -> bool {
        if start_index < 8 {
            return false;
        }

        let mut current_index = start_index as i32 - 4;
        let mut repetitions = 1_usize;
        let zobrist = self.history[start_index as usize];

        let min_index = max(0, (start_index as i32 - halfmove_clock as i32));

        while current_index >= min_index {
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

#[cfg(test)]
mod test {
    use crate::inkayaku::zobrist_history::ZobristHistory;

    #[test]
    fn test() {
        let mut history = ZobristHistory::default();
        history.set(0, 123);
        history.set(1, 4312);
        history.set(2, 1);
        history.set(3, 2);
        history.set(4, 3);
        history.set(5, 4);
        history.set(6, 1);
        history.set(7, 2);
        history.set(8, 3);
        history.set(9, 4);
        history.set(10, 1);

        assert!(history.is_threefold_repetition(10, 8));
        assert!(!history.is_threefold_repetition(10, 7));
        assert!(!history.is_threefold_repetition(10, 6));
    }
}
