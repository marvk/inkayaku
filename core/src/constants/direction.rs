#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Direction {
    pub delta_file: i32,
    pub delta_rank: i32,
}

impl Direction {
    pub const fn from_deltas(d_file: i32, d_rank: i32) -> Self {
        Self { delta_file: d_file, delta_rank: d_rank }
    }

    pub const fn from_directions(first: Self, second: Self) -> Self {
        Self { delta_file: first.delta_file + second.delta_file, delta_rank: first.delta_rank + second.delta_rank }
    }

    pub const NORTH: Self = Self::from_deltas(0, -1);
    pub const EAST: Self = Self::from_deltas(1, 0);
    pub const SOUTH: Self = Self::from_deltas(0, 1);
    pub const WEST: Self = Self::from_deltas(-1, 0);

    pub const NORTH_EAST: Self = Self::from_directions(Self::NORTH, Self::EAST);
    pub const SOUTH_EAST: Self = Self::from_directions(Self::SOUTH, Self::EAST);
    pub const SOUTH_WEST: Self = Self::from_directions(Self::SOUTH, Self::WEST);
    pub const NORTH_WEST: Self = Self::from_directions(Self::NORTH, Self::WEST);

    pub const NORTH_NORTH_EAST: Self = Self::from_directions(Self::NORTH, Self::NORTH_EAST);
    pub const EAST_NORTH_EAST: Self = Self::from_directions(Self::EAST, Self::NORTH_EAST);
    pub const EAST_SOUTH_EAST: Self = Self::from_directions(Self::EAST, Self::SOUTH_EAST);
    pub const SOUTH_SOUTH_EAST: Self = Self::from_directions(Self::SOUTH, Self::SOUTH_EAST);
    pub const SOUTH_SOUTH_WEST: Self = Self::from_directions(Self::SOUTH, Self::SOUTH_WEST);
    pub const WEST_SOUTH_WEST: Self = Self::from_directions(Self::WEST, Self::SOUTH_WEST);
    pub const WEST_NORTH_WEST: Self = Self::from_directions(Self::WEST, Self::NORTH_WEST);
    pub const NORTH_NORTH_WEST: Self = Self::from_directions(Self::NORTH, Self::NORTH_WEST);

    pub const ORTHOGONAL_DIRECTIONS: [Self; 4] = [Self::NORTH, Self::EAST, Self::SOUTH, Self::WEST];
    pub const DIAGONAL_DIRECTIONS: [Self; 4] = [Self::NORTH_EAST, Self::SOUTH_EAST, Self::SOUTH_WEST, Self::NORTH_WEST];
    pub const CARDINAL_DIRECTIONS: [Self; 8] = [Self::NORTH, Self::EAST, Self::SOUTH, Self::WEST, Self::NORTH_EAST, Self::SOUTH_EAST, Self::SOUTH_WEST, Self::NORTH_WEST];
    pub const KNIGHT_DIRECTIONS: [Self; 8] = [Self::NORTH_NORTH_EAST, Self::EAST_NORTH_EAST, Self::EAST_SOUTH_EAST, Self::SOUTH_SOUTH_EAST, Self::SOUTH_SOUTH_WEST, Self::WEST_SOUTH_WEST, Self::WEST_NORTH_WEST, Self::NORTH_NORTH_WEST];
}

#[cfg(test)]
mod tests {
    use crate::constants::direction::Direction;
    use crate::constants::square::Square;

    #[test]
    #[ignore]
    fn test() {
        let mut occupancy = 0;

        for x in Direction::KNIGHT_DIRECTIONS {
            occupancy |= Square::D4.translate(&x).unwrap().mask;
        }

        println!("{}", occupancy);
    }
}
