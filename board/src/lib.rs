extern crate core;

use marvk_chess_core::constants::square::Square;

use crate::board::constants::{BISHOP, KING, KNIGHT, NO_PIECE, OccupancyBits, PAWN, PieceBits, QUEEN, ROOK, SquareMaskBits, SquareShiftBits};
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
        PAWN => "p",
        KNIGHT => "n",
        BISHOP => "b",
        ROOK => "r",
        QUEEN => "q",
        KING => "k",
        NO_PIECE => "",
        _ => "",
    }.to_string()
}

pub fn square_to_string(square_shift_bits: SquareShiftBits) -> String {
    Square::SQUARES.get(square_shift_bits as usize).map(|s| s.fen()).unwrap_or_else(|| "-".to_string())
}

pub fn move_to_san(mv: &Move) -> String {
    mv.san()
}

#[inline(always)]
pub fn mask_and_shift_from_lowest_one_bit(u: OccupancyBits) -> (SquareMaskBits, SquareShiftBits) {
    let shift = u.trailing_zeros();
    (1 << shift, shift)
}
