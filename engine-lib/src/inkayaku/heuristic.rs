use marvk_chess_board::board::{Bitboard, PlayerState};
use marvk_chess_board::board::constants::{BISHOP, BLACK, GameStageBits, KING, KNIGHT, LATE, MID, OccupancyBits, PAWN, QUEEN, ROOK, WHITE};
use marvk_chess_board::mask_and_shift_from_lowest_one_bit;

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
    fn evaluate(&self, bitboard: &Bitboard, legal_moves_remaining: bool) -> i32 {
        if legal_moves_remaining {
            if bitboard.halfmove_clock >= Self::MAX_HALF_MOVES as u32 {
                self.draw_score()
            } else {
                self.evaluate_ongoing(bitboard)
            }
        } else {
            match (bitboard.is_current_in_check(), bitboard.turn) {
                (true, color) if color == WHITE => self.loss_score() + bitboard.fullmove_clock as i32,
                (true, color) if color == BLACK => self.win_score() - bitboard.fullmove_clock as i32,
                _ => self.draw_score(),
            }
        }
    }

    fn evaluate_ongoing(&self, bitboard: &Bitboard) -> i32;
}

pub struct SimpleHeuristic;

impl SimpleHeuristic {
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

    const WHITE_KING_TABLE_MID: [i32; 64] = [
        -30, -40, -40, -50, -50, -40, -40, -30,
        -30, -40, -40, -50, -50, -40, -40, -30,
        -30, -40, -40, -50, -50, -40, -40, -30,
        -30, -40, -40, -50, -50, -40, -40, -30,
        -20, -30, -30, -40, -40, -30, -30, -20,
        -10, -20, -20, -20, -20, -20, -20, -10,
         20,  20,   0,   0,   0,   0,  20,  20,
         20,  30,  10,   0,   0,  10,  30,  20,
    ];

    const WHITE_QUEEN_TABLE_MID: [i32; 64] = [
        -20, -10, -10,  -5,  -5, -10, -10, -20,
        -10,   0,   0,   0,   0,   0,   0, -10,
        -10,   0,   5,   5,   5,   5,   0, -10,
         -5,   0,   5,   5,   5,   5,   0,  -5,
          0,   0,   5,   5,   5,   5,   0,  -5,
        -10,   5,   5,   5,   5,   5,   0, -10,
        -10,   0,   5,   0,   0,   0,   0, -10,
        -20, -10, -10,  -5,  -5, -10, -10, -20,
    ];

    const WHITE_ROOK_TABLE_MID: [i32; 64] = [
          0,   0,   0,   0,   0,   0,   0,   0,
          5,  10,  10,  10,  10,  10,  10,   5,
         -5,   0,   0,   0,   0,   0,   0,  -5,
         -5,   0,   0,   0,   0,   0,   0,  -5,
         -5,   0,   0,   0,   0,   0,   0,  -5,
         -5,   0,   0,   0,   0,   0,   0,  -5,
         -5,   0,   0,   0,   0,   0,   0,  -5,
          0,   0,   0,   5,   5,   0,   0,   0,
    ];

    const WHITE_BISHOP_TABLE_MID: [i32; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20,
        -10,   0,   0,   0,   0,   0,   0, -10,
        -10,   0,   5,  10,  10,   5,   0, -10,
        -10,   5,   5,  10,  10,   5,   5, -10,
        -10,   0,  10,  10,  10,  10,   0, -10,
        -10,  10,  10,  10,  10,  10,  10, -10,
        -10,   5,   0,   0,   0,   0,   5, -10,
        -20, -10, -10, -10, -10, -10, -10, -20,
    ];

    const WHITE_KNIGHT_TABLE_MID: [i32; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50,
        -40, -20,   0,   0,   0,   0, -20, -40,
        -30,   0,  10,  15,  15,  10,   0, -30,
        -30,   5,  15,  20,  20,  15,   5, -30,
        -30,   0,  15,  20,  20,  15,   0, -30,
        -30,   5,  10,  15,  15,  10,   5, -30,
        -40, -20,   0,   5,   5,   0, -20, -40,
        -50, -40, -30, -30, -30, -30, -40, -50,
    ];

    const WHITE_PAWN_TABLE_MID: [i32; 64] = [
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

    const WHITE_TABLES: [[[i32; 64]; 6]; 3] = [
        [Self::WHITE_PAWN_TABLE_MID, Self::WHITE_KNIGHT_TABLE_MID, Self::WHITE_BISHOP_TABLE_MID, Self::WHITE_ROOK_TABLE_MID, Self::WHITE_QUEEN_TABLE_MID, Self::WHITE_KING_TABLE_MID],
        [Self::WHITE_PAWN_TABLE_MID, Self::WHITE_KNIGHT_TABLE_MID, Self::WHITE_BISHOP_TABLE_MID, Self::WHITE_ROOK_TABLE_MID, Self::WHITE_QUEEN_TABLE_MID, Self::WHITE_KING_TABLE_MID],
        [Self::WHITE_PAWN_TABLE_MID, Self::WHITE_KNIGHT_TABLE_MID, Self::WHITE_BISHOP_TABLE_MID, Self::WHITE_ROOK_TABLE_MID, Self::WHITE_QUEEN_TABLE_MID, Self::WHITE_KING_TABLE_LATE],
    ];

    const BLACK_TABLES: [[[i32; 64]; 6]; 3] = Self::mirror_and_flip_sign(Self::WHITE_TABLES);

    const fn mirror_and_flip_sign(tables: [[[i32; 64]; 6]; 3]) -> [[[i32; 64]; 6]; 3] {
        const fn mirror_inner(table: [i32; 64]) -> [i32; 64] {
            let mut result = [0; 64];

            let mut rank = 0;
            while rank < 8 {
                let mut file = 0;
                while file < 8 {
                    result[8 * (8 - rank - 1) + file] = table[8 * rank + file] * -1;
                    file += 1;
                }
                rank += 1;
            }

            result
        }

        const fn mirror_middle(table: [[i32; 64]; 6]) -> [[i32; 64]; 6] {
            [mirror_inner(table[0]),
                mirror_inner(table[1]),
                mirror_inner(table[2]),
                mirror_inner(table[3]),
                mirror_inner(table[4]),
                mirror_inner(table[5]),
            ]
        }

        [mirror_middle(tables[0]),
            mirror_middle(tables[1]),
            mirror_middle(tables[2]), ]
    }

    fn piece_value(state: &PlayerState) -> i32 {
        (state.queens.count_ones() * Self::QUEEN_VALUE +
            state.rooks.count_ones() * Self::ROOK_VALUE +
            state.bishops.count_ones() * Self::BISHOP_VALUE +
            state.knights.count_ones() * Self::KNIGHT_VALUE +
            state.pawns.count_ones() * Self::PAWN_VALUE) as i32
    }

    fn game_stage(&self, board: &Bitboard) -> GameStageBits {
        let white_has_queens = board.white.queens != 0;
        let black_has_queens = board.black.queens != 0;

        let white_has_one_or_fewer_minor_pieces = (board.white.knights | board.white.bishops).count_ones() <= 1;
        let black_has_one_or_fewer_minor_pieces = (board.black.knights | board.black.bishops).count_ones() <= 1;

        let white_has_queens_but_one_or_fewer_minor_pieces = white_has_queens && white_has_one_or_fewer_minor_pieces;
        let black_has_queens_but_one_or_fewer_minor_pieces = black_has_queens && black_has_one_or_fewer_minor_pieces;

        if (!white_has_queens && !black_has_queens)
            || (white_has_queens_but_one_or_fewer_minor_pieces && !black_has_queens)
            || (black_has_queens_but_one_or_fewer_minor_pieces && !white_has_queens)
            || (white_has_one_or_fewer_minor_pieces && black_has_one_or_fewer_minor_pieces) {
            LATE
        } else {
            MID
        }
    }

    fn piece_square_value(&self, board: &Bitboard) -> i32 {
        let stage = self.game_stage(board);

        // println!("stage {}", stage);

        let white_sum = self.piece_square_sum_for_player(&board.white, &Self::WHITE_TABLES[stage]);
        let black_sum = self.piece_square_sum_for_player(&board.black, &Self::BLACK_TABLES[stage]);

        white_sum + black_sum
    }

    fn piece_square_sum_for_player(&self, player: &PlayerState, tables: &[[i32; 64]; 6]) -> i32 {
        self.piece_square_sum(player.pawns, &tables[PAWN as usize - 1])
            + self.piece_square_sum(player.knights, &tables[KNIGHT as usize - 1])
            + self.piece_square_sum(player.bishops, &tables[BISHOP as usize - 1])
            + self.piece_square_sum(player.rooks, &tables[ROOK as usize - 1])
            + self.piece_square_sum(player.queens, &tables[QUEEN as usize - 1])
            + self.piece_square_sum(player.kings, &tables[KING as usize - 1])
    }

    fn piece_square_sum(&self, mut occupancy: OccupancyBits, values: &[i32; 64]) -> i32 {
        let mut sum = 0;

        while occupancy != 0 {
            let (mask, shift) = mask_and_shift_from_lowest_one_bit(occupancy);
            occupancy &= !mask;
            sum += values[shift as usize];
        }

        sum
    }
}

impl Heuristic for SimpleHeuristic {
    fn evaluate_ongoing(&self, bitboard: &Bitboard) -> i32 {
        let my_sum = Self::piece_value(&bitboard.white);
        let their_sum = Self::piece_value(&bitboard.black);
        let psv = self.piece_square_value(bitboard);
        // println!("{}", my_sum);
        // println!("{}", their_sum);
        // println!("{}", psv);

        my_sum - their_sum + psv
    }
}

#[cfg(test)]
mod test {
    use marvk_chess_board::board::Bitboard;
    use marvk_chess_core::fen::{Fen, FEN_STARTPOS};

    use crate::inkayaku::heuristic::{Heuristic, SimpleHeuristic};

    #[test]
    fn test_neutral_psv() {
        let bitboard = Bitboard::new(&FEN_STARTPOS);
        let sut = SimpleHeuristic {};
        let actual_psv = sut.piece_square_value(&bitboard);
        assert_eq!(actual_psv, 0);
    }

    #[test]
    fn evaluate() {
        println!("{}", SimpleHeuristic {}.evaluate(&Bitboard::new(&Fen::new("rn2k2r/ppp2ppp/8/3pPP2/3P1q2/P1KB4/P1P4P/3R2N1 b kq - 0 14").unwrap()), true));
        println!("{}", SimpleHeuristic {}.evaluate(&Bitboard::new(&Fen::new("rn2k2r/ppp2ppp/8/3pPP2/3P1q2/P1KB4/P1P4P/3R2N1 w kq - 0 14").unwrap()), true));
    }
}
