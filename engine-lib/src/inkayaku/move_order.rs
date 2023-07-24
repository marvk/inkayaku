use std::cmp::Reverse;

use marvk_chess_board::board::Move;

pub trait MoveOrder {
    fn sort(&self, moves: &mut Vec<Move>, pv_move: Option<Move>);
}

#[derive(Default)]
pub struct MvvLvaMoveOrder;

impl MvvLvaMoveOrder {
    #[inline(always)]
    fn eval(mv: &Move) -> i32 {
        mv.mvvlva
    }

    #[inline(always)]
    fn pv_move_bonus(pv_move: Option<Move>, mv: &Move) -> i32 {
        pv_move.filter(|pv_move| pv_move.bits == mv.bits).map(|_| 100000).unwrap_or(0)
    }
}

impl MoveOrder for MvvLvaMoveOrder {
    fn sort(&self, moves: &mut Vec<Move>, pv_move: Option<Move>) {
        // moves.shuffle(&mut thread_rng());
        moves.sort_by_key(|mv| Reverse(Self::eval(mv) + Self::pv_move_bonus(pv_move, mv)));
    }
}

#[cfg(test)]
mod tests {
    use marvk_chess_board::board::Bitboard;
    use marvk_chess_core::fen::Fen;

    use crate::inkayaku::move_order::{MoveOrder, MvvLvaMoveOrder};

    #[test]
    #[ignore]
    fn print_move_order() {
        let mut bitboard = Bitboard::new(&Fen::new("k7/8/8/8/5q2/6Pp/7Q/K7 w - - 0 1").unwrap());
        let mut moves = bitboard.generate_legal_moves();

        let order = MvvLvaMoveOrder {};

        order.sort(&mut moves, None);

        for mv in moves {
            println!("{}", mv.to_pgn_string(&mut bitboard).unwrap());
        }
    }
}
