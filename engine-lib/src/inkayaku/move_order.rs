use std::cmp::Reverse;

use rand::prelude::SliceRandom;
use rand::thread_rng;

use marvk_chess_board::board::Move;

pub trait MoveOrder {
    fn sort(&self, moves: &mut Vec<Move>);
}

pub struct MvvLvaMoveOrder;

impl MvvLvaMoveOrder {
    #[inline(always)]
    fn eval(mv: &Move) -> i32 {
        mv.1
    }
}

impl MoveOrder for MvvLvaMoveOrder {
    fn sort(&self, moves: &mut Vec<Move>) {
        moves.shuffle(&mut thread_rng());
        moves.sort_by_key(|mv| Reverse(Self::eval(mv)));
    }
}

#[cfg(test)]
mod tests {
    use marvk_chess_board::board::Bitboard;
    use marvk_chess_core::fen::{Fen};
    use crate::inkayaku::move_order::{MoveOrder, MvvLvaMoveOrder};

    #[test]
    #[ignore]
    fn print_move_order() {
        let mut bitboard = Bitboard::new(&Fen::new("k7/8/8/8/5q2/6Pp/7Q/K7 w - - 0 1").unwrap());
        let mut  moves = bitboard.generate_legal_moves();

        let order = MvvLvaMoveOrder {};

        order.sort(&mut moves);

        for mv in moves {
            println!("{}", mv.to_san_string(&mut bitboard));
        }
    }
}
