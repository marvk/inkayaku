extern crate core;

use marvk_chess_board::board::Move;
use marvk_chess_uci::uci::UciMove;

pub mod inkayaku;

fn move_into_uci_move(mv: Move) -> UciMove {
    match mv.structs() {
        (source, target, None) => UciMove::new(source, target),
        (source, target, Some(piece)) => UciMove::new_with_promotion(source, target, piece),
    }
}
