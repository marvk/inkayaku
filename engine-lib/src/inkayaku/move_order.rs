use std::cmp::Reverse;
use rand::prelude::SliceRandom;
use rand::thread_rng;

use marvk_chess_board::board::Move;

pub trait MoveOrder {
    fn sort(&self, moves: &mut Vec<Move>);
}

pub struct MvvLvaMoveOrder;

impl MoveOrder for MvvLvaMoveOrder {
    fn sort(&self, moves: &mut Vec<Move>) {
        moves.shuffle(&mut thread_rng());
        moves.sort_by_key(|mv| Reverse(mv.1));
    }
}
