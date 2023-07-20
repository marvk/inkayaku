use crate::constants::color::Color;
use crate::constants::colored_piece::ColoredPiece;

#[non_exhaustive]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Piece {
    pub name: &'static str,
    pub fen: char,
    pub index: u8,
}

impl Piece {
    pub const PAWN: Self = Self { name: "Pawn", fen: 'p', index: 1 };
    pub const KNIGHT: Self = Self { name: "Knight", fen: 'n', index: 2 };
    pub const BISHOP: Self = Self { name: "Bishop", fen: 'b', index: 3 };
    pub const ROOK: Self = Self { name: "Rook", fen: 'r', index: 4 };
    pub const QUEEN: Self = Self { name: "Queen", fen: 'q', index: 5 };
    pub const KING: Self = Self { name: "King", fen: 'k', index: 6 };

    pub fn as_color(&self, color: Color) -> ColoredPiece {
        match color {
            Color::WHITE => self.as_white(),
            Color::BLACK => self.as_black(),
            _ => panic!(),
        }
    }

    pub fn as_black(&self) -> ColoredPiece {
        match *self {
            Self::PAWN => ColoredPiece::BLACK_PAWN,
            Self::KNIGHT => ColoredPiece::BLACK_KNIGHT,
            Self::BISHOP => ColoredPiece::BLACK_BISHOP,
            Self::ROOK => ColoredPiece::BLACK_ROOK,
            Self::QUEEN => ColoredPiece::BLACK_QUEEN,
            Self::KING => ColoredPiece::BLACK_KING,
            _ => panic!(),
        }
    }

    pub fn as_white(&self) -> ColoredPiece {
        match *self {
            Self::PAWN => ColoredPiece::WHITE_PAWN,
            Self::KNIGHT => ColoredPiece::WHITE_KNIGHT,
            Self::BISHOP => ColoredPiece::WHITE_BISHOP,
            Self::ROOK => ColoredPiece::WHITE_ROOK,
            Self::QUEEN => ColoredPiece::WHITE_QUEEN,
            Self::KING => ColoredPiece::WHITE_KING,
            _ => panic!(),
        }
    }

    pub fn uci_name(&self) -> char {
        self.as_black().fen
    }

    pub const PIECES: [Self; 6] = [Self::PAWN, Self::KNIGHT, Self::BISHOP, Self::ROOK, Self::QUEEN, Self::KING];

    pub fn by_char(c: char) -> Option<Piece> {
        match c {
            'K' | 'k' => Some(Self::KING),
            'Q' | 'q' => Some(Self::QUEEN),
            'R' | 'r' => Some(Self::ROOK),
            'B' | 'b' => Some(Self::BISHOP),
            'N' | 'n' => Some(Self::KNIGHT),
            'P' | 'p' => Some(Self::PAWN),
            _ => None,
        }
    }

    pub fn by_index(index: usize) -> Option<Piece> {
        match index {
            1 => Some(Self::PAWN),
            2 => Some(Self::KNIGHT),
            3 => Some(Self::BISHOP),
            4 => Some(Self::ROOK),
            5 => Some(Self::QUEEN),
            6 => Some(Self::KING),
            _ => None
        }
    }
}
