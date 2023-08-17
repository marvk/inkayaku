use inkayaku_board::Bitboard;
use inkayaku_board::constants::ZobristHash;

use crate::engine::heuristic::{Heuristic, mirror_and_flip_sign, PieceCounts};
use crate::engine::table::HashTable;

const QUEEN_VALUE: u32 = 900;
const ROOK_VALUE: u32 = 500;
const BISHOP_VALUE: u32 = 330;
const KNIGHT_VALUE: u32 = 320;
const PAWN_VALUE: u32 = 100;

// @formatter:off

const WHITE_KING_TABLE_LATE: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -30,   0,   0,   0,   0, -30, -30,
    -50, -30, -30, -30, -30, -30, -30, -50,
];

const WHITE_KING_TABLE_EARLY: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20,
];

const WHITE_QUEEN_TABLE_EARLY: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

const WHITE_ROOK_TABLE_EARLY: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,   0,   5,   5,   0,   0,   0,
];

const WHITE_BISHOP_TABLE_EARLY: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const WHITE_KNIGHT_TABLE_EARLY: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const WHITE_PAWN_TABLE_EARLY: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  10,  25,  25,  10,   5,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,  -5, -10,   0,   0, -10,  -5,   5,
      5,  10,  10, -20, -20,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

// @formatter:on

const WHITE_TABLES: [[[i32; 64]; 6]; 2] = [
    [WHITE_PAWN_TABLE_EARLY, WHITE_KNIGHT_TABLE_EARLY, WHITE_BISHOP_TABLE_EARLY, WHITE_ROOK_TABLE_EARLY, WHITE_QUEEN_TABLE_EARLY, WHITE_KING_TABLE_EARLY],
    [WHITE_PAWN_TABLE_EARLY, WHITE_KNIGHT_TABLE_EARLY, WHITE_BISHOP_TABLE_EARLY, WHITE_ROOK_TABLE_EARLY, WHITE_QUEEN_TABLE_EARLY, WHITE_KING_TABLE_LATE],
];

const BLACK_TABLES: [[[i32; 64]; 6]; 2] = mirror_and_flip_sign(WHITE_TABLES);

struct PawnEval {}

pub struct ImprovedHeuristic {
    pawn_state_table: HashTable<ZobristHash, PawnEval>,
}

impl ImprovedHeuristic {}

impl Heuristic for ImprovedHeuristic {
    fn evaluate_ongoing(&self, bitboard: &Bitboard, zobrist_pawn_hash: ZobristHash) -> i32 {
        let counts = PieceCounts::count_from(bitboard);

        let taper_factor = taper_factor(&counts);


        todo!()
    }
}

/// Returns the taper factor in `0..=255`, 0 being early game and 255 being end game
fn taper_factor(counts: &PieceCounts) -> u8 {
    const PAWN_PHASE: i32 = 0;
    const KNIGHT_PHASE: i32 = 1;
    const BISHOP_PHASE: i32 = 1;
    const ROOK_PHASE: i32 = 2;
    const QUEEN_PHASE: i32 = 4;
    const TOTAL_PHASE: i32 = PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

    let phase = TOTAL_PHASE
        - counts.pawns() as i32 * PAWN_PHASE
        - counts.knights() as i32 * KNIGHT_PHASE
        - counts.bishops() as i32 * BISHOP_PHASE
        - counts.rooks() as i32 * ROOK_PHASE
        - counts.queens() as i32 * QUEEN_PHASE
        ;

    let result = (phase * 255 + TOTAL_PHASE) / TOTAL_PHASE - 1;

    result.clamp(0, 255) as u8
}
