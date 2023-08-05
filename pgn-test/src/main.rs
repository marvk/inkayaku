use std::cmp::{max, min};
use std::fs::{File, write};
use std::ops::Deref;
use std::str::FromStr;
use std::thread;
use std::time::Instant;

use marvk_chess_board::board::{Bitboard, PlayerState};
use marvk_chess_board::board::constants::{BLACK, ColorBits, DRAW, KING, PAWN, PieceBits, WHITE};
use marvk_chess_board::mask_and_shift_from_lowest_one_bit;
use marvk_chess_core::constants::color::Color;
use marvk_chess_pgn::reader::{PgnRaw, PgnRawParser};

use crate::PgnExclusion::{BlackEloNotAvailable, BlackEloNotParsable, BlackEloTooLow, TimeControlNotAvailable, TimeControlNotParsable, TimeNotParsable, TimeTooLow, WhiteEloNotAvailable, WhiteEloNotParsable, WhiteEloTooLow};

fn main() {
    test();
}


fn test() {
    let file = File::open("Y:\\Data\\lichess_db_standard_rated_2023-07.pgn.zst").unwrap();

    let start = Instant::now();

    let mut parser = PgnRawParser::new(zstd::Decoder::new(file).unwrap());

    let mut t = 0;
    let mut i = 0;

    let mut buckets = TaperPieceCountBucket::default();

    loop {
        match parser.next() {
            Some(Ok(pgn)) => {
                t += 1;
                if filter_pgn(&pgn).is_err() {
                    continue;
                }

                calc(pgn, &mut buckets);

                i += 1;

                if i >= 2_700_000 {
                    break;
                }
                println!("{}/{}", i, t);
            }
            Some(Err(err)) => {
                println!("{:?}", err);
                break;
            }
            _ => { break; }
        }
    }


    let mut str = String::new();


    for result in WHITE..=DRAW {
        for color in marvk_chess_core::constants::color::Color::VALUES {
            for piece in marvk_chess_core::constants::piece::Piece::VALUES {
                for taper_factor in 0..=24 {
                    let z = buckets.get(color.index, piece.index as PieceBits, taper_factor, result);
                    let result = if result == 0 { Some(Color::WHITE) } else if result == 1 { Some(Color::BLACK) } else { None };
                    let result = result.map_or("Draw", |c| c.name);
                    str.push_str(&format!("{:}\t{:}\t{:}\t{:}\t{:?}\n", result, color.name, piece.name, taper_factor, z));
                }
            }
        }
    }

    write("out", str).unwrap();

    dbg!(start.elapsed());
}

#[derive(Debug)]
enum PgnExclusion {
    BlackEloNotAvailable,
    BlackEloNotParsable(String),
    BlackEloTooLow(u32),
    WhiteEloNotAvailable,
    WhiteEloNotParsable(String),
    WhiteEloTooLow(u32),
    TimeControlNotAvailable,
    TimeControlNotParsable(String),
    TimeNotParsable(String),
    TimeTooLow(u32),
}

fn filter_pgn(pgn: &PgnRaw) -> Result<(), PgnExclusion> {
    if let Some(white_elo) = pgn.tag_pairs.get("WhiteElo") {
        if let Ok(white_elo) = u32::from_str(white_elo) {
            if white_elo < 1800 {
                return Err(WhiteEloTooLow(white_elo));
            }
        } else {
            return Err(WhiteEloNotParsable(white_elo.to_string()));
        }
    } else {
        return Err(WhiteEloNotAvailable);
    }

    if let Some(black_elo) = pgn.tag_pairs.get("BlackElo") {
        if let Ok(black_elo) = u32::from_str(black_elo) {
            if black_elo < 1800 {
                return Err(BlackEloTooLow(black_elo));
            }
        } else {
            return Err(BlackEloNotParsable(black_elo.to_string()));
        }
    } else {
        return Err(BlackEloNotAvailable);
    }


    if let Some(time_control) = pgn.tag_pairs.get("TimeControl") {
        if let Some((time, ..)) = time_control.split_once('+') {
            if let Ok(time) = u32::from_str(time) {
                if time < 600 {
                    return Err(TimeTooLow(time));
                }
            } else {
                return Err(TimeNotParsable(time.to_string()));
            }
        } else {
            return Err(TimeControlNotParsable(time_control.to_string()));
        }
    } else {
        return Err(TimeControlNotAvailable);
    }

    Ok(())
}

fn calc(pgn: PgnRaw, buckets: &mut TaperPieceCountBucket) {
    let mut board = Bitboard::default();

    let game_result = pgn.tag_pairs.get("Result").map(|s| s.as_str());
    let result = match game_result {
        Some("0-1") => BLACK,
        Some("1-0") => WHITE,
        Some("1/2-1/2") => DRAW,
        _ => {
            println!("Failed to resolve result {:?}", game_result);
            return;
        }
    };

    for x in &pgn.moves {
        if let Ok(mv) = board.pgn_to_bb(&x.mv) {
            board.make(mv);
            let taper_factor = taper_factor(&board);


            buckets.add(WHITE, &board.white, taper_factor, result);
            buckets.add(BLACK, &board.black, taper_factor, result);
        } else {
            panic!("{:?}\n{:?}", x, pgn);
        }
    }
}

type InternalBuckets = [[[[[u64; 64]; 3]; 6]; 25]; 2];

struct TaperPieceCountBucket {
    buckets: Box<InternalBuckets>,
}

impl Default for TaperPieceCountBucket {
    fn default() -> Self {
        Self { buckets: Box::new([[[[[0; 64]; 3]; 6]; 25]; 2]) }
    }
}

impl TaperPieceCountBucket {
    fn add(&mut self, color: ColorBits, state: &PlayerState, taper_factor: u8, result: ColorBits) {
        for piece in PAWN..=KING {
            let mut occupancy = state.occupancy(piece);

            while occupancy != 0 {
                let (mask, shift) = mask_and_shift_from_lowest_one_bit(occupancy);
                occupancy &= !mask;

                self.buckets[color as usize][taper_factor as usize][(piece - 1) as usize][result as usize][shift as usize] += 1;
            }
        }
    }

    fn get(&self, color: ColorBits, piece: PieceBits, taper_factor: u8, result: ColorBits) -> [u64; 64] {
        self.buckets[color as usize][taper_factor as usize][(piece - 1) as usize][result as usize]
    }
}

/// Returns the taper factor in `0..=24`, 0 being early game and 24 being end game
fn taper_factor(board: &Bitboard) -> u8 {
    const PAWN_PHASE: i32 = 0;
    const KNIGHT_PHASE: i32 = 1;
    const BISHOP_PHASE: i32 = 1;
    const ROOK_PHASE: i32 = 2;
    const QUEEN_PHASE: i32 = 4;
    const TOTAL_PHASE: i32 = PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

    let phase = TOTAL_PHASE
        - (board.white.pawns().count_ones() + board.black.pawns().count_ones()) as i32 * PAWN_PHASE
        - (board.white.knights().count_ones() + board.black.knights().count_ones()) as i32 * KNIGHT_PHASE
        - (board.white.bishops().count_ones() + board.black.bishops().count_ones()) as i32 * BISHOP_PHASE
        - (board.white.rooks().count_ones() + board.black.rooks().count_ones()) as i32 * ROOK_PHASE
        - (board.white.queens().count_ones() + board.black.queens().count_ones()) as i32 * QUEEN_PHASE
        ;

    min(max(phase, 0), TOTAL_PHASE) as u8
}
