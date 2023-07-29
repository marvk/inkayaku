use marvk_chess_board::board::{Bitboard, PlayerState};
use marvk_chess_board::board::constants::{BISHOP, GameStageBits, KING, KNIGHT, LATE, MID, OccupancyBits, PAWN, QUEEN, ROOK, WHITE};
use marvk_chess_board::mask_and_shift_from_lowest_one_bit;
use marvk_chess_uci::uci::Score;
use marvk_chess_uci::uci::Score::Mate;
use crate::inkayaku::heuristic::{Heuristic, mirror_and_flip_sign};

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
    [WHITE_PAWN_TABLE_MID, WHITE_KNIGHT_TABLE_MID, WHITE_BISHOP_TABLE_MID, WHITE_ROOK_TABLE_MID, WHITE_QUEEN_TABLE_MID, WHITE_KING_TABLE_MID],
    [WHITE_PAWN_TABLE_MID, WHITE_KNIGHT_TABLE_MID, WHITE_BISHOP_TABLE_MID, WHITE_ROOK_TABLE_MID, WHITE_QUEEN_TABLE_MID, WHITE_KING_TABLE_MID],
    [WHITE_PAWN_TABLE_MID, WHITE_KNIGHT_TABLE_MID, WHITE_BISHOP_TABLE_MID, WHITE_ROOK_TABLE_MID, WHITE_QUEEN_TABLE_MID, WHITE_KING_TABLE_LATE],
];

const BLACK_TABLES: [[[i32; 64]; 6]; 3] = mirror_and_flip_sign(WHITE_TABLES);

#[derive(Default)]
pub struct SimpleHeuristic;

impl SimpleHeuristic {
    fn piece_value(state: &PlayerState) -> i32 {
        (state.queens().count_ones() * QUEEN_VALUE +
            state.rooks().count_ones() * ROOK_VALUE +
            state.bishops().count_ones() * BISHOP_VALUE +
            state.knights().count_ones() * KNIGHT_VALUE +
            state.pawns().count_ones() * PAWN_VALUE) as i32
    }

    fn game_stage(board: &Bitboard) -> GameStageBits {
        let white_has_queens = board.white.queens() != 0;
        let black_has_queens = board.black.queens() != 0;

        let white_has_one_or_fewer_minor_pieces = (board.white.knights() | board.white.bishops()).count_ones() <= 1;
        let black_has_one_or_fewer_minor_pieces = (board.black.knights() | board.black.bishops()).count_ones() <= 1;

        let white_has_queens_but_one_or_fewer_minor_pieces = white_has_queens && white_has_one_or_fewer_minor_pieces;
        let black_has_queens_but_one_or_fewer_minor_pieces = black_has_queens && black_has_one_or_fewer_minor_pieces;

        #[allow(clippy::nonminimal_bool)]
        if (!white_has_queens && !black_has_queens)
            || (white_has_queens_but_one_or_fewer_minor_pieces && !black_has_queens)
            || (black_has_queens_but_one_or_fewer_minor_pieces && !white_has_queens)
            || (white_has_one_or_fewer_minor_pieces && black_has_one_or_fewer_minor_pieces) {
            LATE
        } else {
            MID
        }
    }

    fn piece_square_value(board: &Bitboard) -> i32 {
        let stage = Self::game_stage(board);

        let white_sum = Self::piece_square_sum_for_player(&board.white, &WHITE_TABLES[stage]);
        let black_sum = Self::piece_square_sum_for_player(&board.black, &BLACK_TABLES[stage]);

        white_sum + black_sum
    }

    fn piece_square_sum_for_player(player: &PlayerState, tables: &[[i32; 64]; 6]) -> i32 {
        Self::piece_square_sum(player.pawns(), &tables[PAWN as usize - 1])
            + Self::piece_square_sum(player.knights(), &tables[KNIGHT as usize - 1])
            + Self::piece_square_sum(player.bishops(), &tables[BISHOP as usize - 1])
            + Self::piece_square_sum(player.rooks(), &tables[ROOK as usize - 1])
            + Self::piece_square_sum(player.queens(), &tables[QUEEN as usize - 1])
            + Self::piece_square_sum(player.kings(), &tables[KING as usize - 1])
    }

    fn piece_square_sum(mut occupancy: OccupancyBits, values: &[i32; 64]) -> i32 {
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
        let psv = Self::piece_square_value(bitboard);

        my_sum - their_sum + psv
    }
}

#[cfg(test)]
mod test {
    use marvk_chess_board::board::Bitboard;
    use crate::inkayaku::heuristic::Heuristic;
    use crate::inkayaku::heuristic::simple::SimpleHeuristic;

    #[test]
    fn test_neutral_psv() {
        let bitboard = Bitboard::default();
        let sut = SimpleHeuristic {};
        let actual_psv = sut.piece_square_value(&bitboard);
        assert_eq!(actual_psv, 0);
    }

    #[test]
    fn evaluate() {
        println!("{}", SimpleHeuristic {}.evaluate(&Bitboard::from_fen_string_unchecked("rn2k2r/ppp2ppp/8/3pPP2/3P1q2/P1KB4/P1P4P/3R2N1 b kq - 0 14"), true));
        println!("{}", SimpleHeuristic {}.evaluate(&Bitboard::from_fen_string_unchecked("rn2k2r/ppp2ppp/8/3pPP2/3P1q2/P1KB4/P1P4P/3R2N1 w kq - 0 14"), true));
    }
}
