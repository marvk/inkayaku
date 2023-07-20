extern crate core;

use marvk_chess_board::board::{Move, MoveStructs};
use marvk_chess_uci::uci::UciMove;

pub mod inkayaku;

fn move_into_uci_move(mv: Move) -> UciMove {
    let MoveStructs { from_square, to_square, promote_to, .. } = MoveStructs::from(mv);

    match promote_to {
        Some(promote_to) => UciMove::new_with_promotion(from_square, to_square, promote_to),
        None => UciMove::new(from_square, to_square),
    }
}
