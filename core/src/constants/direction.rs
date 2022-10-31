#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Direction {
    pub d_file: i32,
    pub d_rank: i32,
}

impl Direction {
    pub const fn from(d_file: i32, d_rank: i32) -> Direction {
        Direction { d_file, d_rank }
    }

    pub const fn from_others(first: Direction, second: Direction) -> Direction {
        Direction { d_file: first.d_file + second.d_file, d_rank: first.d_rank + second.d_rank }
    }

    pub const NORTH: Self = Self::from(0, -1);
    pub const EAST: Self = Self::from(1, 0);
    pub const SOUTH: Self = Self::from(0, 1);
    pub const WEST: Self = Self::from(-1, 0);

    pub const NORTH_EAST: Self = Self::from_others(Self::NORTH, Self::EAST);
    pub const SOUTH_EAST: Self = Self::from_others(Self::SOUTH, Self::EAST);
    pub const SOUTH_WEST: Self = Self::from_others(Self::SOUTH, Self::WEST);
    pub const NORTH_WEST: Self = Self::from_others(Self::NORTH, Self::WEST);

    pub const NORTH_NORTH_EAST: Self = Self::from_others(Self::NORTH, Self::NORTH_EAST);
    pub const EAST_NORTH_EAST: Self = Self::from_others(Self::EAST, Self::NORTH_EAST);
    pub const EAST_SOUTH_EAST: Self = Self::from_others(Self::EAST, Self::SOUTH_EAST);
    pub const SOUTH_SOUTH_EAST: Self = Self::from_others(Self::SOUTH, Self::SOUTH_EAST);
    pub const SOUTH_SOUTH_WEST: Self = Self::from_others(Self::SOUTH, Self::SOUTH_WEST);
    pub const WEST_SOUTH_WEST: Self = Self::from_others(Self::WEST, Self::SOUTH_WEST);
    pub const WEST_NORTH_WEST: Self = Self::from_others(Self::WEST, Self::NORTH_WEST);
    pub const NORTH_NORTH_WEST: Self = Self::from_others(Self::NORTH, Self::NORTH_WEST);

    pub const ORTHOGONAL_DIRECTIONS: [Self; 4] = [Self::NORTH, Self::EAST, Self::SOUTH, Self::WEST, ];
    pub const DIAGONAL_DIRECTIONS: [Self; 4] = [Self::NORTH_EAST, Self::SOUTH_EAST, Self::SOUTH_WEST, Self::NORTH_WEST, ];
    pub const CARDINAL_DIRECTIONS: [Self; 8] = [Self::NORTH, Self::EAST, Self::SOUTH, Self::WEST, Self::NORTH_EAST, Self::SOUTH_EAST, Self::SOUTH_WEST, Self::NORTH_WEST, ];
    pub const KNIGHT_DIRECTIONS: [Self; 8] = [Self::NORTH_NORTH_EAST, Self::EAST_NORTH_EAST, Self::EAST_SOUTH_EAST, Self::SOUTH_SOUTH_EAST, Self::SOUTH_SOUTH_WEST, Self::WEST_SOUTH_WEST, Self::WEST_NORTH_WEST, Self::NORTH_NORTH_WEST, ];
}

#[cfg(test)]
mod tests {
    use crate::constants::direction::Direction;
    use crate::constants::square::Square;

    #[test]
    fn test() {
        let mut occupancy = 0;

        for x in Direction::KNIGHT_DIRECTIONS {
            occupancy |= Square::D4.translate(&x).unwrap().mask;
        }

        println!("{}", occupancy);
    }
}
