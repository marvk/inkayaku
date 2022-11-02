#[non_exhaustive]
#[derive(Eq, PartialEq, Debug)]
pub struct Color {
    pub name: &'static str,
    pub index: u32,
}

impl Color {
    pub const WHITE: Self = Self { name: "White", index: 0 };
    pub const BLACK: Self = Self { name: "Black", index: 1 };

    pub const COLORS: [Self; 2] = [Self::WHITE, Self::BLACK];

    pub fn by_index(index: usize) -> Color {
        match index {
            0 => Self::WHITE,
            1 => Self::BLACK,
            _ => panic!(),
        }
    }
}
