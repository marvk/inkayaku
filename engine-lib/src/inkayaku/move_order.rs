use std::cmp::Reverse;

use marvk_chess_board::Move;

pub trait MoveOrder {
    fn sort(&self, moves: &mut Vec<Move>, pv_move: Option<Move>, transposition_move: Option<Move>, killer_move: Option<Move>);
}

#[derive(Default)]
pub struct MvvLvaMoveOrder;

impl MvvLvaMoveOrder {
    #[inline(always)]
    const fn eval(mv: &Move) -> i32 {
        mv.mvvlva
    }

    #[inline(always)]
    fn move_bonus(mv: &Move, high_value_move: Option<Move>, bonus: i32) -> i32 {
        high_value_move.filter(|pv_move| pv_move.bits == mv.bits).map_or(0, |_| bonus)
    }
}

impl MoveOrder for MvvLvaMoveOrder {
    fn sort(&self, moves: &mut Vec<Move>, pv_move: Option<Move>, transposition_move: Option<Move>, killer_move: Option<Move>) {
        moves.sort_by_key(|mv| Reverse(
            Self::eval(mv)
                + Self::move_bonus(mv, pv_move, 900_000)
                + Self::move_bonus(mv, transposition_move, 800_000)
                + Self::move_bonus(mv, killer_move, 700_000)
        ));
    }
}

#[cfg(test)]
mod tests {
    use marvk_chess_board::Bitboard;

    use crate::inkayaku::move_order::{MoveOrder, MvvLvaMoveOrder};

    #[test]
    #[ignore]
    fn print_move_order() {
        let mut bitboard = Bitboard::from_fen_string_unchecked("k7/8/8/8/5q2/6Pp/7Q/K7 w - - 0 1");
        let mut moves = bitboard.generate_legal_moves();

        let order = MvvLvaMoveOrder {};

        order.sort(&mut moves, None, None, None);

        for mv in moves {
            println!("{}", mv.to_pgn_string(&mut bitboard).unwrap());
        }
    }
}
