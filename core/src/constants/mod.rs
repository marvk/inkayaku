use crate::constants::file::File;
use crate::constants::rank::Rank;

pub mod file;
pub mod rank;
pub mod square;
pub mod piece;
pub mod colored_piece;
pub mod color;
pub mod direction;

pub const fn to_square_index_from_indices(file_index: usize, rank_index: usize) -> usize {
    file_index + rank_index * 8_usize
}

pub const fn to_square_index_from_structs(file: File, rank: Rank) -> usize {
    to_square_index_from_indices(file.index as usize, rank.index as usize)
}
