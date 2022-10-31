use crate::constants::color::Color;
use crate::constants::piece::Piece;

#[non_exhaustive]
#[derive(Eq, Debug)]
pub struct ColoredPiece {
    pub color: Color,
    pub piece: Piece,
    pub fen: char,
    pub utf8_piece: char,
}


impl PartialEq for ColoredPiece {
    fn eq(&self, other: &Self) -> bool {
        self.piece.index == other.piece.index && self.color.index == other.color.index
    }
}

impl ColoredPiece {
    pub const WHITE_PAWN: Self = Self { color: Color::WHITE, piece: Piece::PAWN, fen: 'P', utf8_piece: '♙' };
    pub const BLACK_PAWN: Self = Self { color: Color::BLACK, piece: Piece::PAWN, fen: 'p', utf8_piece: '♟' };
    pub const WHITE_KNIGHT: Self = Self { color: Color::WHITE, piece: Piece::KNIGHT, fen: 'N', utf8_piece: '♘' };
    pub const BLACK_KNIGHT: Self = Self { color: Color::BLACK, piece: Piece::KNIGHT, fen: 'n', utf8_piece: '♞' };
    pub const WHITE_BISHOP: Self = Self { color: Color::WHITE, piece: Piece::BISHOP, fen: 'B', utf8_piece: '♗' };
    pub const BLACK_BISHOP: Self = Self { color: Color::BLACK, piece: Piece::BISHOP, fen: 'b', utf8_piece: '♝' };
    pub const WHITE_ROOK: Self = Self { color: Color::WHITE, piece: Piece::ROOK, fen: 'R', utf8_piece: '♖' };
    pub const BLACK_ROOK: Self = Self { color: Color::BLACK, piece: Piece::ROOK, fen: 'r', utf8_piece: '♜' };
    pub const WHITE_QUEEN: Self = Self { color: Color::WHITE, piece: Piece::QUEEN, fen: 'Q', utf8_piece: '♕' };
    pub const BLACK_QUEEN: Self = Self { color: Color::BLACK, piece: Piece::QUEEN, fen: 'q', utf8_piece: '♛' };
    pub const WHITE_KING: Self = Self { color: Color::WHITE, piece: Piece::KING, fen: 'K', utf8_piece: '♔' };
    pub const BLACK_KING: Self = Self { color: Color::BLACK, piece: Piece::KING, fen: 'k', utf8_piece: '♚' };

    pub const COLORED_PIECES: [Self; 12] = [Self::WHITE_PAWN, Self::BLACK_PAWN, Self::WHITE_KNIGHT, Self::BLACK_KNIGHT, Self::WHITE_BISHOP, Self::BLACK_BISHOP, Self::WHITE_ROOK, Self::BLACK_ROOK, Self::WHITE_QUEEN, Self::BLACK_QUEEN, Self::WHITE_KING, Self::BLACK_KING, ];

    pub fn name(&self) -> String {
        format!("{} {}", self.color.name, self.piece.name)
    }

    pub fn by_index<'a>(color_index: usize, piece_index: usize) -> &'a ColoredPiece {
        &Self::COLORED_PIECES[color_index + piece_index * 2]
    }

    pub fn by<'a>(color: Color, piece: Piece) -> &'a ColoredPiece {
        Self::by_index(color.index as usize, piece.index as usize)
    }
}
