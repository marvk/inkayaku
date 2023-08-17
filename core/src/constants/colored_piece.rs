use std::process::id;
use crate::constants::color::Color;
use crate::constants::piece::Piece;

#[non_exhaustive]
#[derive(Eq, Debug, Clone, Copy)]
pub struct ColoredPiece {
    pub color: Color,
    pub piece: Piece,
    pub fen: char,
    pub utf8_piece: char,
    pub name: &'static str,
}


impl PartialEq for ColoredPiece {
    fn eq(&self, other: &Self) -> bool {
        self.piece.index == other.piece.index && self.color.index == other.color.index
    }
}

impl ColoredPiece {
    pub const WHITE_PAWN: Self = Self { color: Color::WHITE, piece: Piece::PAWN, fen: 'P', utf8_piece: '♙', name: "White Pawn" };
    pub const BLACK_PAWN: Self = Self { color: Color::BLACK, piece: Piece::PAWN, fen: 'p', utf8_piece: '♟', name: "Black Pawn" };
    pub const WHITE_KNIGHT: Self = Self { color: Color::WHITE, piece: Piece::KNIGHT, fen: 'N', utf8_piece: '♘', name: "White Knight" };
    pub const BLACK_KNIGHT: Self = Self { color: Color::BLACK, piece: Piece::KNIGHT, fen: 'n', utf8_piece: '♞', name: "Black Knight" };
    pub const WHITE_BISHOP: Self = Self { color: Color::WHITE, piece: Piece::BISHOP, fen: 'B', utf8_piece: '♗', name: "White Bishop" };
    pub const BLACK_BISHOP: Self = Self { color: Color::BLACK, piece: Piece::BISHOP, fen: 'b', utf8_piece: '♝', name: "Black Bishop" };
    pub const WHITE_ROOK: Self = Self { color: Color::WHITE, piece: Piece::ROOK, fen: 'R', utf8_piece: '♖', name: "White Rook" };
    pub const BLACK_ROOK: Self = Self { color: Color::BLACK, piece: Piece::ROOK, fen: 'r', utf8_piece: '♜', name: "Black Rook" };
    pub const WHITE_QUEEN: Self = Self { color: Color::WHITE, piece: Piece::QUEEN, fen: 'Q', utf8_piece: '♕', name: "White Queen" };
    pub const BLACK_QUEEN: Self = Self { color: Color::BLACK, piece: Piece::QUEEN, fen: 'q', utf8_piece: '♛', name: "Black Queen" };
    pub const WHITE_KING: Self = Self { color: Color::WHITE, piece: Piece::KING, fen: 'K', utf8_piece: '♔', name: "White King" };
    pub const BLACK_KING: Self = Self { color: Color::BLACK, piece: Piece::KING, fen: 'k', utf8_piece: '♚', name: "Black King" };

    pub const VALUES: [Self; 12] = [Self::WHITE_PAWN, Self::BLACK_PAWN, Self::WHITE_KNIGHT, Self::BLACK_KNIGHT, Self::WHITE_BISHOP, Self::BLACK_BISHOP, Self::WHITE_ROOK, Self::BLACK_ROOK, Self::WHITE_QUEEN, Self::BLACK_QUEEN, Self::WHITE_KING, Self::BLACK_KING, ];

    pub fn from_indices(color_index: usize, piece_index: usize) -> Option<Self> {
        if color_index <= 1 && piece_index > 0 && piece_index <= 6 {
            Self::VALUES.get(Self::idx(color_index, piece_index)).copied()
        } else {
            None
        }
    }

    pub const fn from_indices_unchecked(color_index: usize, piece_index: usize) -> Self {
        Self::VALUES[Self::idx(color_index, piece_index)]
    }

    pub fn from_structs(color: Color, piece: Piece) -> Option<ColoredPiece> {
        Self::from_indices(color.index as usize, piece.index as usize)
    }

    const fn idx(color_index: usize, piece_index: usize) -> usize {
        color_index + (piece_index - 1) * 2
    }
}

#[cfg(test)]
mod test {
    use crate::constants::colored_piece::ColoredPiece;

    #[test]
    fn test_from_indices() {
        assert_eq!(ColoredPiece::from_indices(0, 0), None);
        assert_eq!(ColoredPiece::from_indices(1, 0), None);
        assert_eq!(ColoredPiece::from_indices(0, 1), Some(ColoredPiece::WHITE_PAWN));
        assert_eq!(ColoredPiece::from_indices(1, 1), Some(ColoredPiece::BLACK_PAWN));
        assert_eq!(ColoredPiece::from_indices(0, 2), Some(ColoredPiece::WHITE_KNIGHT));
        assert_eq!(ColoredPiece::from_indices(1, 2), Some(ColoredPiece::BLACK_KNIGHT));
        assert_eq!(ColoredPiece::from_indices(0, 3), Some(ColoredPiece::WHITE_BISHOP));
        assert_eq!(ColoredPiece::from_indices(1, 3), Some(ColoredPiece::BLACK_BISHOP));
        assert_eq!(ColoredPiece::from_indices(0, 4), Some(ColoredPiece::WHITE_ROOK));
        assert_eq!(ColoredPiece::from_indices(1, 4), Some(ColoredPiece::BLACK_ROOK));
        assert_eq!(ColoredPiece::from_indices(0, 5), Some(ColoredPiece::WHITE_QUEEN));
        assert_eq!(ColoredPiece::from_indices(1, 5), Some(ColoredPiece::BLACK_QUEEN));
        assert_eq!(ColoredPiece::from_indices(0, 6), Some(ColoredPiece::WHITE_KING));
        assert_eq!(ColoredPiece::from_indices(1, 6), Some(ColoredPiece::BLACK_KING));
        assert_eq!(ColoredPiece::from_indices(2, 0), None);
        assert_eq!(ColoredPiece::from_indices(0, 7), None);
    }
}
