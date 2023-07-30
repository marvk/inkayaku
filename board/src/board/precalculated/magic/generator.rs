use std::cell::RefCell;
use std::collections::HashSet;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use marvk_chess_core::constants::direction::Direction;
use marvk_chess_core::constants::piece::Piece;
use marvk_chess_core::constants::square::Square;
use crate::board::precalculated::magic::magic_hash;

#[derive(Debug, Eq, PartialEq)]
pub struct GeneratorConfiguration {
    pub mask: u64,
    pub magic: u64,
    pub hash_mask: u64,
    pub hash_shift: u32,
    pub attacks: Vec<u64>,
}

impl GeneratorConfiguration {
    pub(crate) const fn new(mask: u64, magic: u64, hash_mask: u64, hash_shift: u32, attacks: Vec<u64>) -> Self {
        Self { mask, magic, hash_mask, hash_shift, attacks }
    }

    #[inline(always)]
    pub fn get_attacks(&self, occupancy: u64) -> u64 {
        unsafe {
            *self.attacks.get_unchecked(self.hash(occupancy))
        }
    }

    #[inline(always)]
    fn hash(&self, occupancy: u64) -> usize {
        magic_hash(self.mask, self.hash_shift, self.hash_mask, self.magic, occupancy)
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


    pub fn generate_all_attacks_with_magic(&self, magic: u64) -> GeneratorConfiguration {
        let mut result = vec![0; self.num_possible_configurations];

        for &occupancy in &self.possible_configurations {
            result[self.hash(magic, occupancy)] = self.generate_attack(occupancy)
        }

        GeneratorConfiguration::new(
            self.mask,
            magic,
            self.hash_mask,
            self.hash_shift,
            result,
        )
    }

    pub fn generate_all_attacks(&self) -> GeneratorConfiguration {
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

            if let Some(square) = current {
                result |= square.mask;
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

        true
    }

    fn hash(&self, magic: u64, occupancy: u64) -> usize {
        magic_hash(self.mask, self.hash_shift, self.hash_mask, magic, occupancy)
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::write;

    use marvk_chess_core::constants::piece::Piece;
    use marvk_chess_core::constants::square::Square;
    use crate::board::precalculated::magic::generator::{GeneratorConfiguration, ConfigurationGenerator};

    fn generate_magic_hashes_for(piece: Piece) -> [u64; 64] {
        Square::SQUARES.iter().map(|&square| { ConfigurationGenerator::new(piece, square).generate_all_attacks().magic }).collect::<Vec<_>>().try_into().unwrap()
    }

    #[test]
    #[ignore]
    fn generate_magics() {
        let rook_magics: [u64; 64] = generate_magic_hashes_for(Piece::ROOK);
        let bishop_magics: [u64; 64] = generate_magic_hashes_for(Piece::BISHOP);

        println!("const BISHOP_MAGICS: [u64; 64] = {:?};", bishop_magics);
        println!("const ROOK_MAGICS: [u64; 64] = {:?};", rook_magics);
    }

    fn generate_magics_for(piece: Piece, magic_hashes: [u64; 64]) -> [GeneratorConfiguration; 64] {
        Square::SQUARES.iter().enumerate().map(|(index, &square)| { ConfigurationGenerator::new(piece, square).generate_all_attacks_with_magic(magic_hashes[index]) }).collect::<Vec<_>>().try_into().unwrap()
    }

    const BISHOP_MAGIC_HASHES: [u64; 64] = [
        54188608189382912, 1396296206707408896, 6199055180890120, 2825753536241664, 9260531269400790080, 576755558925731984, 2378188194532824065, 602575455979520,
        1495204009965912128, 75470482459623680, 54047627968790528, 2450314460565340162, 1152961104759226496, 283678364142101, 361559293667352576, 10414713792126976,
        594510369679016960, 4652218552550818880, 793834682155401794, 9225629473846607873, 2814835700662273, 2346656889477071184, 577590165636270081, 9077570180124683,
        1161093473374212, 1143494783077632, 2882343344338387972, 1126179549544450, 292736724625294337, 40618162865078816, 2307532413191882756, 3396992730677505,
        4613973024089589761, 36171939692687377, 2324042142861230240, 180181370638172288, 76773400483463232, 1134730628203552, 580967240300823553, 4904703676796403785,
        9260531269400790080, 36592374172000512, 2452220734555099136, 2450099072619516164, 324294375058187585, 10487144086899457, 45220716384092416, 4613975837367631938,
        2378188194532824065, 36592374172000512, 282646446874694, 36028797566845088, 1441151933645324320, 5188208687247819265, 54047627968790528, 1396296206707408896,
        602575455979520, 10414713792126976, 9225659179959128064, 288266266148602885, 405364653225681924, 1126175376425216, 1495204009965912128, 54188608189382912
    ];

    const ROOK_MAGICS_HASHES: [u64; 64] = [
        252201717645448328, 666537418043695105, 2449993932256315456, 144132799056583168, 2449964794494590988, 72066394427098112, 144116289875739136, 144115327680135684,
        9223512775417929856, 38351034315178048, 703824889712640, 598415912168611920, 324399945019031680, 14074066665474224, 198299129682657408, 2450521153721139396,
        11673365968343533696, 576602041158221826, 4504151532765200, 9297682534515085312, 1172137669459775744, 1134700832768004, 9223376435037769745, 36030996046446860,
        2322213655445537, 1170940855217824000, 9232388033277329472, 72066392283676801, 11529778038971830304, 14074066665474224, 1153071055368228865, 5480934081413792004,
        3675043673726259332, 2382404340422803521, 9799850383500124163, 576619101313574912, 8798248911872, 6896145527735296, 2535684334159888, 72343468302663809,
        3476797076839890944, 4936160833585037312, 9376529612856688656, 9024795805024264, 288511919982182420, 4647785253447401732, 73333371140571202, 144116984513691659,
        9223512775417929856, 70395051934080, 9232388033277329472, 1190164169039577216, 578712862495637632, 182958769560360448, 2378059010369756160, 144116289875739136,
        4724558240564658690, 4724558240564658690, 448738221900033, 4612249209436155970, 1153203032331143697, 1166713812867612673, 576759843105409036, 288230692435954434
    ];

    #[test]
    #[ignore]
    fn generate_const() {
        let rook_magics = generate_magics_for(Piece::ROOK, ROOK_MAGICS_HASHES);
        let bishop_magics = generate_magics_for(Piece::BISHOP, BISHOP_MAGIC_HASHES);

        let rook = format!("const ROOK_MAGICS: Magics = Magics([{}]);", generate_string(rook_magics));
        let bishop = format!("const BISHOP_MAGICS: Magics = Magics([{}]);", generate_string(bishop_magics));

        let result = format!("{}\n{}", rook, bishop);

        dbg!(env::current_dir().ok());

        write("out", result).ok();
    }

    fn generate_string(x: [GeneratorConfiguration; 64]) -> String {
        x.iter().map(|conf| {
            let array_string = format!("vec![{}]", conf.attacks.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(", "));

            format!("Configuration::new({}, {}, {}, {}, {})", conf.mask, conf.magic, conf.hash_mask, conf.hash_shift, array_string)
        }).collect::<Vec<_>>().join(",\n")
    }
}

