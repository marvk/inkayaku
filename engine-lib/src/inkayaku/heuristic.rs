use marvk_chess_board::board::{Bitboard, PlayerState};
use marvk_chess_board::board::constants::{BLACK, WHITE, ZobristHash};
use marvk_chess_uci::uci::Score;
use marvk_chess_uci::uci::Score::{Centipawn, Mate};

pub mod simple;
pub mod improved;

pub trait Heuristic {
    const MAX_FULL_MOVES: i32 = 1 << 20;
    const MAX_HALF_MOVES: u32 = 50;
    #[inline(always)]
    fn win_score(&self) -> i32 { 1 << 24 }
    #[inline(always)]
    fn loss_score(&self) -> i32 { -self.win_score() }
    #[inline(always)]
    fn draw_score(&self) -> i32 { 0 }
    #[inline(always)]
    fn is_checkmate(&self, value: i32) -> bool {
        value > self.win_score() - Self::MAX_FULL_MOVES || value < self.loss_score() + Self::MAX_FULL_MOVES
    }
    fn evaluate(&self, bitboard: &Bitboard, zobrist_pawn_hash: ZobristHash, legal_moves_remaining: bool) -> i32 {
        if legal_moves_remaining {
            if bitboard.halfmove_clock >= Self::MAX_HALF_MOVES {
                self.draw_score()
            } else {
                self.evaluate_ongoing(bitboard, zobrist_pawn_hash)
            }
        } else {
            match (bitboard.is_current_in_check(), bitboard.turn) {
                (true, color) if color == WHITE => self.loss_score() + bitboard.fullmove_clock as i32,
                (true, color) if color == BLACK => self.win_score() - bitboard.fullmove_clock as i32,
                _ => self.draw_score(),
            }
        }
    }
    fn score_from_value(&self, value: i32, bitboard: &Bitboard) -> Score {
        if value.abs() > self.win_score() / 2 {
            let offset = i32::from(value > 0 && bitboard.turn == WHITE);
            let mate_in = (self.win_score() - value.abs() - bitboard.fullmove_clock as i32 + offset) * value.signum();
            Mate { mate_in }
        } else {
            Centipawn { score: value }
        }
    }

    fn evaluate_ongoing(&self, bitboard: &Bitboard, zobrist_pawn_hash: ZobristHash) -> i32;
}

const fn mirror_and_flip_sign<const M: usize, const T: usize>(tables: [[[i32; 64]; M]; T]) -> [[[i32; 64]; M]; T] {
    const fn mirror_inner(table: [i32; 64]) -> [i32; 64] {
        let mut result = [0; 64];

        let mut rank = 0;
        while rank < 8 {
            let mut file = 0;
            while file < 8 {
                result[8 * (8 - rank - 1) + file] = -table[8 * rank + file];
                file += 1;
            }
            rank += 1;
        }

        result
    }

    const fn mirror_middle<const M: usize>(table: [[i32; 64]; M]) -> [[i32; 64]; M] {
        let mut result = [[0; 64]; M];

        let mut men = 0;
        while men < M {
            result[men] = mirror_inner(table[men]);
            men += 1;
        }

        result
    }

    let mut result = [[[0; 64]; M]; T];

    let mut table = 0;
    while table < T {
        result[table] = mirror_middle(tables[table]);
        table += 1;
    }

    result
}

struct PieceCount {
    pawns: u32,
    knights: u32,
    bishops: u32,
    rooks: u32,
    queens: u32,
}

impl PieceCount {
    pub const fn count_from(player_state: &PlayerState) -> Self {
        Self {
            pawns: player_state.pawns().count_ones(),
            knights: player_state.knights().count_ones(),
            bishops: player_state.bishops().count_ones(),
            rooks: player_state.rooks().count_ones(),
            queens: player_state.queens().count_ones(),
        }
    }
}

struct PieceCounts {
    white: PieceCount,
    black: PieceCount,
}

impl PieceCounts {
    pub const fn count_from(bitboard: &Bitboard) -> Self {
        Self {
            white: PieceCount::count_from(&bitboard.white),
            black: PieceCount::count_from(&bitboard.black),
        }
    }

    const fn pawns(&self) -> u32 { self.white.pawns + self.black.pawns }
    const fn knights(&self) -> u32 { self.white.knights + self.black.knights }
    const fn bishops(&self) -> u32 { self.white.bishops + self.black.bishops }
    const fn rooks(&self) -> u32 { self.white.rooks + self.black.rooks }
    const fn queens(&self) -> u32 { self.white.queens + self.black.queens }
}
