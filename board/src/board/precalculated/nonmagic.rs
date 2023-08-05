use marvk_chess_core::constants::direction::Direction;
use marvk_chess_core::constants::square::Square;
use crate::board::constants::SquareShiftBits;

pub type Nonmagics = [u64; 64];

pub trait UnsafeNonmagicsExt {
    fn get_attacks(&self, square_shift: SquareShiftBits) -> u64;
}

impl UnsafeNonmagicsExt for Nonmagics {
    #[inline(always)]
    fn get_attacks(&self, square_shift: SquareShiftBits) -> u64 {
        unsafe {
            *self.get_unchecked(square_shift as usize)
        }
    }
}

pub const KING_NONMAGICS: Nonmagics = build_nonmagics(&Direction::CARDINAL_DIRECTIONS);
pub const KNIGHT_NONMAGICS: Nonmagics = build_nonmagics(&Direction::KNIGHT_DIRECTIONS);
pub const WHITE_PAWN_NONMAGICS: Nonmagics = build_nonmagics(&[Direction::NORTH_WEST, Direction::NORTH_EAST]);
pub const BLACK_PAWN_NONMAGICS: Nonmagics = build_nonmagics(&[Direction::SOUTH_WEST, Direction::SOUTH_EAST]);

const fn build_nonmagics(directions: &[Direction]) -> Nonmagics {
    let mut result = [0; 64];

    let mut square_shift = 0;
    while square_shift < 64 {
        result[square_shift] = attack_occupations(square_shift, directions);

        square_shift += 1;
    }

    result
}

const fn attack_occupations(square_shift: usize, directions: &[Direction]) -> u64 {
    let square = Square::VALUES[square_shift];

    let mut result: u64 = 0;

    let mut i = 0;
    while i < directions.len() {
        if let Some(square) = square.translate(&directions[i]) {
            result |= square.mask;
        }

        i += 1;
    }

    result
}
