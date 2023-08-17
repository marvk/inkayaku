use std::cmp::max;
use inkayaku_board::constants::ZobristHash;

pub struct ZobristHistory {
    history: [ZobristHash; 5000],
}

impl ZobristHistory {
    pub fn set(&mut self, index: u16, zobrist_hash: ZobristHash) {
        self.history[index as usize] = zobrist_hash;
    }

    pub fn count_repetitions(&self, start_index: u16, halfmove_clock: u16) -> usize {
        if start_index < 4 {
            return 0;
        }

        let mut current_index = start_index as i32 - 4;
        let mut repetitions = 1_usize;
        let zobrist = self.history[start_index as usize];

        let min_index = max(0, start_index as i32 - halfmove_clock as i32);

        while current_index >= min_index {
            let current_zobrist = self.history[current_index as usize];
            if current_zobrist == zobrist {
                repetitions += 1;

                if repetitions >= 3 {
                    return 3;
                }
            }

            current_index -= 2;
        }

        repetitions
    }
}

impl Default for ZobristHistory {
    fn default() -> Self {
        Self { history: [0; 5000] }
    }
}

#[cfg(test)]
mod test {
    use crate::engine::zobrist_history::ZobristHistory;

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

        assert_eq!(history.count_repetitions(10, 8), 3);
        assert_ne!(history.count_repetitions(10, 7), 3);
        assert_ne!(history.count_repetitions(10, 6), 3);
    }
}
