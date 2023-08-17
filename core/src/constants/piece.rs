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

    pub const VALUES: [Self; 6] = [Self::PAWN, Self::KNIGHT, Self::BISHOP, Self::ROOK, Self::QUEEN, Self::KING];

    pub const fn to_color(&self, color: Color) -> ColoredPiece {
        match color.index {
            0 => self.to_white(),
            1 => self.to_black(),
            _ => panic!(),
        }
    }

    pub const fn to_black(&self) -> ColoredPiece {
        ColoredPiece::from_indices_unchecked(Color::BLACK.index as usize, self.index as usize)
    }

    pub const fn to_white(&self) -> ColoredPiece {
        ColoredPiece::from_indices_unchecked(Color::WHITE.index as usize, self.index as usize)
    }

    pub const fn from_char(c: char) -> Option<Self> {
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

    #[allow(clippy::unwrap_used)]
    pub fn from_char_unchecked(c: char) -> Self {
        Self::from_char(c).unwrap()
    }

    pub const fn from_index(index: usize) -> Option<Self> {
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

    pub const fn from_index_unchecked(index: usize) -> Self {
        Self::VALUES[index - 1]
    }
}
