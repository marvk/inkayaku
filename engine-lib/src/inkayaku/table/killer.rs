use std::cmp::min;
use marvk_chess_board::board::Move;

#[derive(Default)]
pub struct KillerTable {
    table: Vec<Move>,
}

impl KillerTable {
    pub fn clear(&mut self) {
        self.table.clear()
    }

    pub fn age(&mut self, plys: usize) {
        self.table.drain(0..min(plys, self.table.len()));
    }

    pub fn put(&mut self, depth: usize, mv: Move) {
        self.table.resize(depth + 1, Move::default());
        self.table[depth] = mv;
    }

    pub fn get(&self, depth: usize) -> Option<Move> {
        self.table.get(depth).filter(|mv| mv.bits != 0).copied()
    }
}
