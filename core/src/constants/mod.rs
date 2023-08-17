
mod file;
mod rank;
mod square;
mod piece;
mod colored_piece;
mod color;
mod direction;

pub use file::File;
pub use rank::Rank;
pub use square::Square;
pub use piece::Piece;
pub use colored_piece::ColoredPiece;
pub use color::Color;
pub use direction::Direction;

pub const fn to_square_index_from_indices(file_index: usize, rank_index: usize) -> usize {
    file_index + rank_index * 8_usize
}

pub const fn to_square_index_from_structs(file: File, rank: Rank) -> usize {
    to_square_index_from_indices(file.index as usize, rank.index as usize)
}
