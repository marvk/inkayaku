extern crate core;

use marvk_chess_core::constants::square::Square;

use crate::board::constants::{BISHOP, KING, KNIGHT, NO_PIECE, OccupancyBits, PAWN, PieceBits, QUEEN, ROOK, SquareShiftBits};
use crate::board::Move;

pub mod board;

pub fn occupancy_to_string(occupancy: OccupancyBits) -> String {
    let reversed = occupancy.reverse_bits();
    let mask = 0b11111111;
    let mut result = String::new();

    for i in (0..8).rev() {
        let row = (reversed >> (8 * i)) & mask;

        for j in (0..8).rev() {
            let cur = if (1 << j) & row != 0 {
                '1'
            } else {
                '·'
            };

            result.push_str(&format!(" {} ", cur));
        }

        result.push('\n');
    }

    result
}


pub fn piece_to_string(piece_bits: PieceBits) -> String {
    match piece_bits {
        NO_PIECE => "",
        PAWN => "p",
        KNIGHT => "n",
        BISHOP => "b",
        ROOK => "r",
        QUEEN => "q",
        KING => "k",
        _ => "???",
    }.to_string()
}

pub fn square_to_string(square_shift_bits: SquareShiftBits) -> String {
    Square::SQUARES.get(square_shift_bits as usize).map(|s| s.fen()).unwrap_or_else(|| "-".to_string())
}

pub fn move_to_san_reduced(mv: &Move) -> String {
    format!("{}{}{}", square_to_string(mv.get_source_square()), square_to_string(mv.get_target_square()), piece_to_string(mv.get_promotion_piece()))
}

pub fn highest_one_bit(u: u64) -> u64 {
    u & 0x8000000000000000_u64.rotate_right(u.leading_zeros())
}
