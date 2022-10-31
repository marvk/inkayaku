#[non_exhaustive]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct File {
    pub fen: char,
    pub index: u8,
}

impl File {
    pub const FILE_A: Self = Self { fen: 'a', index: 0 };
    pub const FILE_B: Self = Self { fen: 'b', index: 1 };
    pub const FILE_C: Self = Self { fen: 'c', index: 2 };
    pub const FILE_D: Self = Self { fen: 'd', index: 3 };
    pub const FILE_E: Self = Self { fen: 'e', index: 4 };
    pub const FILE_F: Self = Self { fen: 'f', index: 5 };
    pub const FILE_G: Self = Self { fen: 'g', index: 6 };
    pub const FILE_H: Self = Self { fen: 'h', index: 7 };

    pub const FILES: [Self; 8] = [Self::FILE_H, Self::FILE_G, Self::FILE_F, Self::FILE_E, Self::FILE_D, Self::FILE_C, Self::FILE_B, Self::FILE_A, ];

    pub fn by_index<'a>(index: usize) -> &'a Self {
        &Self::FILES[index]
    }
}
