use std::cmp::Reverse;

use marvk_chess_board::board::Move;

pub trait MoveOrder {
    fn sort(&self, moves: &mut Vec<Move>);
}

pub struct MvvLvaMoveOrder;

impl MoveOrder for MvvLvaMoveOrder {
    fn sort(&self, moves: &mut Vec<Move>) {
        moves.sort_by_key(|mv| Reverse(mv.0))
    }
}
