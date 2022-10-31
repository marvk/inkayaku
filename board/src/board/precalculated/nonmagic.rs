use lazy_static::lazy_static;

use marvk_chess_core::constants::direction::Direction;
use marvk_chess_core::constants::square::Square;

use crate::board::constants::SquareShiftBits;

pub struct Nonmagics([u64; 64]);

lazy_static! {
    pub static ref KING_NONMAGICS: Nonmagics = Nonmagics::new(&Direction::CARDINAL_DIRECTIONS);
    pub static ref KNIGHT_NONMAGICS: Nonmagics = Nonmagics::new(&Direction::KNIGHT_DIRECTIONS);
    pub static ref WHITE_PAWN_NONMAGICS: Nonmagics = Nonmagics::new(&[Direction::NORTH_WEST, Direction::NORTH_EAST]);
    pub static ref BLACK_PAWN_NONMAGICS: Nonmagics = Nonmagics::new(&[Direction::SOUTH_WEST, Direction::SOUTH_EAST]);
}

impl Nonmagics {
    pub fn get_attacks(&self, square: SquareShiftBits) -> u64 {
        self.0[square as usize]
    }

    fn new(directions: &[Direction]) -> Nonmagics {
        let attack_occupations =
            (0..64)
                .into_iter()
                .map(|square_shift| {
                    Self::attack_occupations(square_shift, directions)
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

        Nonmagics(attack_occupations)
    }

    fn attack_occupations(square_shift: usize, directions: &[Direction]) -> u64 {
        let square = Square::SQUARES[square_shift];

        directions
            .iter()
            .filter_map(|direction| square.translate(direction))
            .fold(0_u64, |acc, square| { acc | square.mask })
    }
}
