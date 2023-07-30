use lazy_static::lazy_static;

use marvk_chess_core::constants::direction::Direction;
use marvk_chess_core::constants::square::Square;

use crate::board::constants::SquareShiftBits;

pub struct Nonmagics([u64; 64]);

pub const KING_NONMAGICS: Nonmagics = Nonmagics::new(&Direction::CARDINAL_DIRECTIONS);
pub const KNIGHT_NONMAGICS: Nonmagics = Nonmagics::new(&Direction::KNIGHT_DIRECTIONS);
pub const WHITE_PAWN_NONMAGICS: Nonmagics = Nonmagics::new(&[Direction::NORTH_WEST, Direction::NORTH_EAST]);
pub const BLACK_PAWN_NONMAGICS: Nonmagics = Nonmagics::new(&[Direction::SOUTH_WEST, Direction::SOUTH_EAST]);

impl Nonmagics {
    #[inline(always)]
    pub fn get_attacks(&self, square: SquareShiftBits) -> u64 {
        self.0[square as usize]
    }

    const fn new(directions: &[Direction]) -> Self {
        let mut result = [0; 64];

        let mut square_shift = 0;
        while square_shift < 64 {
            result[square_shift] = Self::attack_occupations(square_shift, directions);

            square_shift += 1;
        }

        Self(result)
    }

    const fn attack_occupations(square_shift: usize, directions: &[Direction]) -> u64 {
        let square = Square::SQUARES[square_shift];


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
}
