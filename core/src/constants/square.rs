use std::fmt::{Debug, Formatter};

use crate::constants::{to_square_index_from_indices, to_square_index_from_structs};
use crate::constants::direction::Direction;

use super::file::File;
use super::rank::Rank;

#[non_exhaustive]
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Square {
    pub file: File,
    pub rank: Rank,
    pub mask: u64,
    pub shift: u32,
}

impl Debug for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Square({}{})",
            self.file.fen,
            self.rank.fen,
        )
    }
}

impl Square {
    pub const fn new(file: File, rank: Rank) -> Self {
        let index = to_square_index_from_structs(file, rank) as u32;

        Self {
            file,
            rank,
            mask: 1 << index,
            shift: index,
        }
    }

    pub const A8: Self = Self::new(File::FILE_A, Rank::RANK_8);
    pub const B8: Self = Self::new(File::FILE_B, Rank::RANK_8);
    pub const C8: Self = Self::new(File::FILE_C, Rank::RANK_8);
    pub const D8: Self = Self::new(File::FILE_D, Rank::RANK_8);
    pub const E8: Self = Self::new(File::FILE_E, Rank::RANK_8);
    pub const F8: Self = Self::new(File::FILE_F, Rank::RANK_8);
    pub const G8: Self = Self::new(File::FILE_G, Rank::RANK_8);
    pub const H8: Self = Self::new(File::FILE_H, Rank::RANK_8);
    pub const A7: Self = Self::new(File::FILE_A, Rank::RANK_7);
    pub const B7: Self = Self::new(File::FILE_B, Rank::RANK_7);
    pub const C7: Self = Self::new(File::FILE_C, Rank::RANK_7);
    pub const D7: Self = Self::new(File::FILE_D, Rank::RANK_7);
    pub const E7: Self = Self::new(File::FILE_E, Rank::RANK_7);
    pub const F7: Self = Self::new(File::FILE_F, Rank::RANK_7);
    pub const G7: Self = Self::new(File::FILE_G, Rank::RANK_7);
    pub const H7: Self = Self::new(File::FILE_H, Rank::RANK_7);
    pub const A6: Self = Self::new(File::FILE_A, Rank::RANK_6);
    pub const B6: Self = Self::new(File::FILE_B, Rank::RANK_6);
    pub const C6: Self = Self::new(File::FILE_C, Rank::RANK_6);
    pub const D6: Self = Self::new(File::FILE_D, Rank::RANK_6);
    pub const E6: Self = Self::new(File::FILE_E, Rank::RANK_6);
    pub const F6: Self = Self::new(File::FILE_F, Rank::RANK_6);
    pub const G6: Self = Self::new(File::FILE_G, Rank::RANK_6);
    pub const H6: Self = Self::new(File::FILE_H, Rank::RANK_6);
    pub const A5: Self = Self::new(File::FILE_A, Rank::RANK_5);
    pub const B5: Self = Self::new(File::FILE_B, Rank::RANK_5);
    pub const C5: Self = Self::new(File::FILE_C, Rank::RANK_5);
    pub const D5: Self = Self::new(File::FILE_D, Rank::RANK_5);
    pub const E5: Self = Self::new(File::FILE_E, Rank::RANK_5);
    pub const F5: Self = Self::new(File::FILE_F, Rank::RANK_5);
    pub const G5: Self = Self::new(File::FILE_G, Rank::RANK_5);
    pub const H5: Self = Self::new(File::FILE_H, Rank::RANK_5);
    pub const A4: Self = Self::new(File::FILE_A, Rank::RANK_4);
    pub const B4: Self = Self::new(File::FILE_B, Rank::RANK_4);
    pub const C4: Self = Self::new(File::FILE_C, Rank::RANK_4);
    pub const D4: Self = Self::new(File::FILE_D, Rank::RANK_4);
    pub const E4: Self = Self::new(File::FILE_E, Rank::RANK_4);
    pub const F4: Self = Self::new(File::FILE_F, Rank::RANK_4);
    pub const G4: Self = Self::new(File::FILE_G, Rank::RANK_4);
    pub const H4: Self = Self::new(File::FILE_H, Rank::RANK_4);
    pub const A3: Self = Self::new(File::FILE_A, Rank::RANK_3);
    pub const B3: Self = Self::new(File::FILE_B, Rank::RANK_3);
    pub const C3: Self = Self::new(File::FILE_C, Rank::RANK_3);
    pub const D3: Self = Self::new(File::FILE_D, Rank::RANK_3);
    pub const E3: Self = Self::new(File::FILE_E, Rank::RANK_3);
    pub const F3: Self = Self::new(File::FILE_F, Rank::RANK_3);
    pub const G3: Self = Self::new(File::FILE_G, Rank::RANK_3);
    pub const H3: Self = Self::new(File::FILE_H, Rank::RANK_3);
    pub const A2: Self = Self::new(File::FILE_A, Rank::RANK_2);
    pub const B2: Self = Self::new(File::FILE_B, Rank::RANK_2);
    pub const C2: Self = Self::new(File::FILE_C, Rank::RANK_2);
    pub const D2: Self = Self::new(File::FILE_D, Rank::RANK_2);
    pub const E2: Self = Self::new(File::FILE_E, Rank::RANK_2);
    pub const F2: Self = Self::new(File::FILE_F, Rank::RANK_2);
    pub const G2: Self = Self::new(File::FILE_G, Rank::RANK_2);
    pub const H2: Self = Self::new(File::FILE_H, Rank::RANK_2);
    pub const A1: Self = Self::new(File::FILE_A, Rank::RANK_1);
    pub const B1: Self = Self::new(File::FILE_B, Rank::RANK_1);
    pub const C1: Self = Self::new(File::FILE_C, Rank::RANK_1);
    pub const D1: Self = Self::new(File::FILE_D, Rank::RANK_1);
    pub const E1: Self = Self::new(File::FILE_E, Rank::RANK_1);
    pub const F1: Self = Self::new(File::FILE_F, Rank::RANK_1);
    pub const G1: Self = Self::new(File::FILE_G, Rank::RANK_1);
    pub const H1: Self = Self::new(File::FILE_H, Rank::RANK_1);

    pub fn fen(&self) -> String {
        format!("{}{}", self.file.fen, self.rank.fen)
    }

    pub const VALUES: [Self; 64] = [
        Self::A8, Self::B8, Self::C8, Self::D8, Self::E8, Self::F8, Self::G8, Self::H8,
        Self::A7, Self::B7, Self::C7, Self::D7, Self::E7, Self::F7, Self::G7, Self::H7,
        Self::A6, Self::B6, Self::C6, Self::D6, Self::E6, Self::F6, Self::G6, Self::H6,
        Self::A5, Self::B5, Self::C5, Self::D5, Self::E5, Self::F5, Self::G5, Self::H5,
        Self::A4, Self::B4, Self::C4, Self::D4, Self::E4, Self::F4, Self::G4, Self::H4,
        Self::A3, Self::B3, Self::C3, Self::D3, Self::E3, Self::F3, Self::G3, Self::H3,
        Self::A2, Self::B2, Self::C2, Self::D2, Self::E2, Self::F2, Self::G2, Self::H2,
        Self::A1, Self::B1, Self::C1, Self::D1, Self::E1, Self::F1, Self::G1, Self::H1,
    ];

    pub fn by_chars(file: char, rank: char) -> Option<Self> {
        let file = (file as usize) - ('a' as usize);
        let i = rank.to_digit(10)?;
        let rank = 8_u32.wrapping_sub(i) as usize;
        Self::by_indices(file, rank)
    }

    pub const fn by_indices(file_index: usize, rank_index: usize) -> Option<Self> {
        if file_index > 7 || rank_index > 7 {
            return None;
        }

        Self::by_index(to_square_index_from_indices(file_index, rank_index))
    }

    // TODO grab from array
    pub const fn by_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::A8),
            1 => Some(Self::B8),
            2 => Some(Self::C8),
            3 => Some(Self::D8),
            4 => Some(Self::E8),
            5 => Some(Self::F8),
            6 => Some(Self::G8),
            7 => Some(Self::H8),
            8 => Some(Self::A7),
            9 => Some(Self::B7),
            10 => Some(Self::C7),
            11 => Some(Self::D7),
            12 => Some(Self::E7),
            13 => Some(Self::F7),
            14 => Some(Self::G7),
            15 => Some(Self::H7),
            16 => Some(Self::A6),
            17 => Some(Self::B6),
            18 => Some(Self::C6),
            19 => Some(Self::D6),
            20 => Some(Self::E6),
            21 => Some(Self::F6),
            22 => Some(Self::G6),
            23 => Some(Self::H6),
            24 => Some(Self::A5),
            25 => Some(Self::B5),
            26 => Some(Self::C5),
            27 => Some(Self::D5),
            28 => Some(Self::E5),
            29 => Some(Self::F5),
            30 => Some(Self::G5),
            31 => Some(Self::H5),
            32 => Some(Self::A4),
            33 => Some(Self::B4),
            34 => Some(Self::C4),
            35 => Some(Self::D4),
            36 => Some(Self::E4),
            37 => Some(Self::F4),
            38 => Some(Self::G4),
            39 => Some(Self::H4),
            40 => Some(Self::A3),
            41 => Some(Self::B3),
            42 => Some(Self::C3),
            43 => Some(Self::D3),
            44 => Some(Self::E3),
            45 => Some(Self::F3),
            46 => Some(Self::G3),
            47 => Some(Self::H3),
            48 => Some(Self::A2),
            49 => Some(Self::B2),
            50 => Some(Self::C2),
            51 => Some(Self::D2),
            52 => Some(Self::E2),
            53 => Some(Self::F2),
            54 => Some(Self::G2),
            55 => Some(Self::H2),
            56 => Some(Self::A1),
            57 => Some(Self::B1),
            58 => Some(Self::C1),
            59 => Some(Self::D1),
            60 => Some(Self::E1),
            61 => Some(Self::F1),
            62 => Some(Self::G1),
            63 => Some(Self::H1),
            _ => None,
        }
    }

    pub fn by(file: &File, rank: &Rank) -> Self {
        #[allow(clippy::unwrap_used)]
        Self::by_indices(file.index as usize, rank.index as usize).unwrap()
    }

    pub const fn translate(&self, direction: &Direction) -> Option<Self> {
        let file = self.file.index as i32 + direction.d_file;
        let rank = self.rank.index as i32 + direction.d_rank;

        if file < 0 || rank < 0 {
            None
        } else {
            Self::by_indices(file as usize, rank as usize)
        }
    }
}


