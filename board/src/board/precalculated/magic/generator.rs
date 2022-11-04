use std::cell::RefCell;
use std::collections::HashSet;


use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use marvk_chess_core::constants::direction::Direction;
use marvk_chess_core::constants::piece::Piece;
use marvk_chess_core::constants::square::Square;

#[derive(Debug, Eq, PartialEq)]
pub struct Configuration {
    pub mask: u64,
    pub magic: u64,
    pub hash_mask: u64,
    pub hash_shift: u32,
    pub attacks: Vec<u64>,
}

impl Configuration {
    #[inline(always)]
    pub fn get_attacks(&self, occupancy: u64) -> u64 {
        self.attacks[self.hash(occupancy)]
    }

    #[inline(always)]
    fn hash(&self, occupancy: u64) -> usize {
        hash(self.mask, self.hash_shift, self.hash_mask, self.magic, occupancy)
    }
}

pub struct ConfigurationGenerator {
    square: Square,
    directions: [Direction; 4],
    mask: u64,
    possible_configurations: Vec<u64>,
    num_possible_configurations: usize,
    hash_mask: u64,
    hash_shift: u32,
    magic_generator: MagicGenerator<StdRng>,
}

#[allow(dead_code)]
impl ConfigurationGenerator {
    pub fn new(piece: Piece, square: Square) -> Self {
        let directions = match piece {
            Piece::BISHOP => Direction::DIAGONAL_DIRECTIONS,
            Piece::ROOK => Direction::ORTHOGONAL_DIRECTIONS,
            _ => panic!(),
        };

        let relevant_squares = Self::relevant_squares(&square, directions);
        let num_relevant_squares = relevant_squares.len();
        let num_possible_configurations = 2_usize.pow(num_relevant_squares as u32);

        let mask = {
            let mut result = 0_u64;
            for &square in &relevant_squares {
                result |= square.mask;
            }
            result
        };

        let hash_mask = (1 << num_relevant_squares) - 1;
        let hash_shift = 64 - num_relevant_squares as u32;

        let possible_configurations = {
            let mut result = Vec::with_capacity(num_possible_configurations);

            for set_bits in 0..num_possible_configurations {
                let mut current = 0;

                for (hash_index, relevant_square) in relevant_squares.iter().enumerate() {
                    let mask = 1 << hash_index;
                    if (set_bits & mask) != 0 {
                        current |= relevant_square.mask
                    }
                }

                result.push(current);
            }

            result
        };

        Self {
            square,
            directions,
            mask,
            possible_configurations,
            num_possible_configurations,
            hash_mask,
            hash_shift,
            magic_generator: MagicGenerator::new(SeedableRng::from_seed([0; 32])),
        }
    }


    pub fn generate_all_attacks_with_magic(&self, magic: u64) -> Configuration {
        let mut result = vec![0; self.num_possible_configurations];

        for &occupancy in &self.possible_configurations {
            result[self.hash(magic, occupancy)] = self.generate_attack(occupancy)
        }

        Configuration {
            mask: self.mask,
            magic,
            hash_mask: self.hash_mask,
            hash_shift: self.hash_shift,
            attacks: result,
        }
    }

    pub fn generate_all_attacks(&self) -> Configuration {
        self.generate_all_attacks_with_magic(self.find_magic())
    }

    fn generate_attack(&self, occupancy: u64) -> u64 {
        let mut result = 0_u64;

        for direction in &self.directions {
            let mut current = self.square.translate(direction);

            while current.is_some() && (occupancy & current.unwrap().mask) == 0 {
                result |= current.unwrap().mask;
                current = current.unwrap().translate(direction);
            }

            if current.is_some() {
                result |= current.unwrap().mask;
            }
        }

        result
    }

    fn relevant_squares(origin: &Square, directions: [Direction; 4]) -> Vec<Square> {
        let mut result: Vec<Square> = Vec::new();

        for direction in directions {
            let mut maybe_current = origin.translate(&direction);

            while maybe_current.is_some() {
                let current = maybe_current.unwrap();
                maybe_current = current.translate(&direction);

                if maybe_current.is_some() {
                    result.push(current)
                }
            }
        }

        result
    }

    fn find_magic(&self) -> u64 {
        let mut set = HashSet::new();
        loop {
            let candidate = self.magic_generator.generate_magic_candidate();

            if self.is_valid_magic(&mut set, candidate) {
                return candidate;
            }

            set.clear()
        }
    }

    fn is_valid_magic(&self, set: &mut HashSet<usize>, candidate: u64) -> bool {
        for &occupancy in &self.possible_configurations {
            if !set.insert(self.hash(candidate, occupancy)) {
                return false;
            }
        }

        return true;
    }

    fn hash(&self, magic: u64, occupancy: u64) -> usize {
        hash(self.mask, self.hash_shift, self.hash_mask, magic, occupancy)
    }
}

struct MagicGenerator<T: Rng> {
    rng: RefCell<T>,
}

impl<T: Rng> MagicGenerator<T> {
    pub fn new(rng: T) -> Self {
        Self { rng: RefCell::new(rng) }
    }

    fn generate_magic_candidate(&self) -> u64 {
        let x: [u64; 4] = self.rng.borrow_mut().gen();

        x[0] & x[1] & x[2]
    }
}

#[inline(always)]
fn hash(mask: u64, hash_shift: u32, hash_mask: u64, magic: u64, occupancy: u64) -> usize {
    let i1 = occupancy & mask;
    let i2 = i1.overflowing_mul(magic).0;
    let i3 = i2 >> hash_shift;
    let i4 = i3 & hash_mask;
    i4 as usize
}

#[cfg(test)]
mod tests {
    use marvk_chess_core::constants::piece::Piece;
    use marvk_chess_core::constants::square::Square;

    use crate::board::precalculated::magic::generator::ConfigurationGenerator;

    #[test]
    fn test() {
        let generator = ConfigurationGenerator::new(Piece::ROOK, Square::A2);

        let attacks = generator.generate_all_attacks();
        println!("{:?}", attacks);
    }
}
