#[non_exhaustive]
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Color {
    pub name: &'static str,
    pub index: u32,
}

impl Color {
    pub const WHITE: Self = Self { name: "White", index: 0 };
    pub const BLACK: Self = Self { name: "Black", index: 1 };

    pub const VALUES: [Self; 2] = [Self::WHITE, Self::BLACK];

    pub fn from_index(index: usize) -> Option<Self> {
        Self::VALUES.get(index).copied()
    }

    pub const fn from_index_unchecked(index: usize) -> Self {
        Self::VALUES[index]
    }
}

#[cfg(test)]
mod test {
    use crate::constants::color::Color;

    #[test]
    fn test_from_index() {
        assert_eq!(Color::from_index(0), Some(Color::WHITE));
        assert_eq!(Color::from_index(1), Some(Color::BLACK));
        assert_eq!(Color::from_index(2), None);
    }
}
