#[non_exhaustive]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Rank {
    pub fen: char,
    pub index: u8,
}

impl Rank {
    pub const RANK_8: Self = Self { fen: '8', index: 0 };
    pub const RANK_7: Self = Self { fen: '7', index: 1 };
    pub const RANK_6: Self = Self { fen: '6', index: 2 };
    pub const RANK_5: Self = Self { fen: '5', index: 3 };
    pub const RANK_4: Self = Self { fen: '4', index: 4 };
    pub const RANK_3: Self = Self { fen: '3', index: 5 };
    pub const RANK_2: Self = Self { fen: '2', index: 6 };
    pub const RANK_1: Self = Self { fen: '1', index: 7 };

    pub const RANKS: [Self; 8] = [Self::RANK_1, Self::RANK_2, Self::RANK_3, Self::RANK_4, Self::RANK_5, Self::RANK_6, Self::RANK_7, Self::RANK_8];

    pub const fn by_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::RANK_8),
            1 => Some(Self::RANK_7),
            2 => Some(Self::RANK_6),
            3 => Some(Self::RANK_5),
            4 => Some(Self::RANK_4),
            5 => Some(Self::RANK_3),
            6 => Some(Self::RANK_2),
            7 => Some(Self::RANK_1),
            _ => None,
        }
    }

    pub const fn by_index_unchecked(index: usize) -> Self {
        Self::RANKS[8 - index - 1]
    }
}
