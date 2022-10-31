use crate::constants::color::Color;
use crate::constants::colored_piece::ColoredPiece;

#[non_exhaustive]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Piece {
    pub name: &'static str,
    pub index: u8,
}

impl Piece {
    pub const PAWN: Self = Self { name: "Pawn", index: 0 };
    pub const KNIGHT: Self = Self { name: "Knight", index: 1 };
    pub const BISHOP: Self = Self { name: "Bishop", index: 2 };
    pub const ROOK: Self = Self { name: "Rook", index: 3 };
    pub const QUEEN: Self = Self { name: "Queen", index: 4 };
    pub const KING: Self = Self { name: "King", index: 5 };

    pub fn as_color(&self, color: Color) -> ColoredPiece {
        match color {
            Color::BLACK => self.as_black(),
            Color::WHITE => self.as_black(),
            _ => panic!(),
        }
    }

    pub fn as_black(&self) -> ColoredPiece {
        match self {
            &Self::PAWN => ColoredPiece::BLACK_PAWN,
            &Self::KNIGHT => ColoredPiece::BLACK_KNIGHT,
            &Self::BISHOP => ColoredPiece::BLACK_BISHOP,
            &Self::ROOK => ColoredPiece::BLACK_ROOK,
            &Self::QUEEN => ColoredPiece::BLACK_QUEEN,
            &Self::KING => ColoredPiece::BLACK_KING,
            _ => panic!(),
        }
    }

    pub fn as_white(&self) -> ColoredPiece {
        match self {
            &Self::PAWN => ColoredPiece::WHITE_PAWN,
            &Self::KNIGHT => ColoredPiece::WHITE_KNIGHT,
            &Self::BISHOP => ColoredPiece::WHITE_BISHOP,
            &Self::ROOK => ColoredPiece::WHITE_ROOK,
            &Self::QUEEN => ColoredPiece::WHITE_QUEEN,
            &Self::KING => ColoredPiece::WHITE_KING,
            _ => panic!(),
        }
    }

    pub fn uci_name(&self) -> char {
        self.as_black().fen
    }

    pub const PIECES: [Self; 6] = [Self::PAWN, Self::KNIGHT, Self::BISHOP, Self::ROOK, Self::QUEEN, Self::KING];

    pub fn by_index<'a>(index: usize) -> &'a Piece {
        &Self::PIECES[index]
    }
}
