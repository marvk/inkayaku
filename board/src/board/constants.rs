#![allow(dead_code)]

use marvk_chess_core::constants::to_square_index_from_indices;

pub type ColorBits = u8;
pub type SquareShiftBits = u32;
pub type SquareMaskBits = u64;
pub type PieceBits = u64;
pub type MaskBits = u64;
pub type ShiftBits = u32;
pub type OccupancyBits = u64;

pub const WHITE: ColorBits = 0;
pub const BLACK: ColorBits = 1;

// MSB . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . LSB
//
// xxxxxxxxxxxx xxx xxxxxx xxxxxx xxxxxxxxxxxx x xxxxxx xxxxxx x x x x x x xxx xxx
// ___UNUSED___  |     |       |         |     |   |      |   | | | | | |  |   |
//               |     |       |         |     |   |      |   | | | | | |  |   |
//               |     |       |         |     |   |      |   | | | | | |  |    --> Piece moved
//               |     |       |         |     |   |      |   | | | | | |  |
//               |     |       |         |     |   |      |   | | | | | |   ------> Piece attacked
//               |     |       |         |     |   |      |   | | | | | |
//               |     |       |         |     |   |      |   | | | | |  ---------> Self lost king side castle
//               |     |       |         |     |   |      |   | | | | |
//               |     |       |         |     |   |      |   | | | |  -----------> Self lost queen side castle
//               |     |       |         |     |   |      |   | | | |
//               |     |       |         |     |   |      |   | | |  -------------> Opponent lost king side castle
//               |     |       |         |     |   |      |   | | |
//               |     |       |         |     |   |      |   | |  ---------------> Opponent lost queen side castle
//               |     |       |         |     |   |      |   | |
//               |     |       |         |     |   |      |   |  -----------------> Is castle move
//               |     |       |         |     |   |      |   |
//               |     |       |         |     |   |      |    -------------------> Is en passant attack
//               |     |       |         |     |   |      |
//               |     |       |         |     |   |       -----------------------> Source square
//               |     |       |         |     |   |
//               |     |       |         |     |    ------------------------------> Target square
//               |     |       |         |     |
//               |     |       |         |      ------------------------------------> Halfmove reset
//               |     |       |         |
//               |     |       |          ------------------------------------------> Previous halfmove
//               |     |       |
//               |     |        ----------------------------------------------------> Previous en passant square } Technically you can use files
//               |     |
//               |      ------------------------------------------------------------> Next en passant square     } and get that information from the pawn move
//               |
//                ------------------------------------------------------------------> Promotion Piece

pub const NO_PIECE: PieceBits = 0;
pub const PAWN: PieceBits = 1;
pub const KNIGHT: PieceBits = 2;
pub const BISHOP: PieceBits = 3;
pub const ROOK: PieceBits = 4;
pub const QUEEN: PieceBits = 5;
pub const KING: PieceBits = 6;

pub const PIECE_MOVED_MASK: MaskBits = 0b111;
pub const PIECE_ATTACKED_MASK: MaskBits = 0b111000;
pub const SELF_LOST_KING_SIDE_CASTLE_MASK: MaskBits = 0b1000000;
pub const SELF_LOST_QUEEN_SIDE_CASTLE_MASK: MaskBits = 0b10000000;
pub const OPPONENT_LOST_KING_SIDE_CASTLE_MASK: MaskBits = 0b100000000;
pub const OPPONENT_LOST_QUEEN_SIDE_CASTLE_MASK: MaskBits = 0b1000000000;
pub const CASTLE_MOVE_MASK: MaskBits = 0b10000000000;
pub const EN_PASSANT_ATTACK_MASK: MaskBits = 0b100000000000;
pub const SOURCE_SQUARE_MASK: MaskBits = 0b111111000000000000;
pub const TARGET_SQUARE_MASK: MaskBits = 0b111111000000000000000000;
pub const HALFMOVE_RESET_MASK: MaskBits = 0b1000000000000000000000000;
pub const PREVIOUS_HALFMOVE_MASK: MaskBits = 0b1111111111110000000000000000000000000;
pub const PREVIOUS_EN_PASSANT_SQUARE_MASK: MaskBits = 0b1111110000000000000000000000000000000000000;
pub const NEXT_EN_PASSANT_SQUARE_MASK: MaskBits = 0b1111110000000000000000000000000000000000000000000;
pub const PROMOTION_PIECE_MASK: MaskBits = 0b1110000000000000000000000000000000000000000000000000;

pub const PIECE_MOVED_SHIFT: ShiftBits = PIECE_MOVED_MASK.trailing_zeros();
pub const PIECE_ATTACKED_SHIFT: ShiftBits = PIECE_ATTACKED_MASK.trailing_zeros();
pub const SELF_LOST_KING_SIDE_CASTLE_SHIFT: ShiftBits = SELF_LOST_KING_SIDE_CASTLE_MASK.trailing_zeros();
pub const SELF_LOST_QUEEN_SIDE_CASTLE_SHIFT: ShiftBits = SELF_LOST_QUEEN_SIDE_CASTLE_MASK.trailing_zeros();
pub const OPPONENT_LOST_KING_SIDE_CASTLE_SHIFT: ShiftBits = OPPONENT_LOST_KING_SIDE_CASTLE_MASK.trailing_zeros();
pub const OPPONENT_LOST_QUEEN_SIDE_CASTLE_SHIFT: ShiftBits = OPPONENT_LOST_QUEEN_SIDE_CASTLE_MASK.trailing_zeros();
pub const CASTLE_MOVE_SHIFT: ShiftBits = CASTLE_MOVE_MASK.trailing_zeros();
pub const EN_PASSANT_ATTACK_SHIFT: ShiftBits = EN_PASSANT_ATTACK_MASK.trailing_zeros();
pub const SOURCE_SQUARE_SHIFT: ShiftBits = SOURCE_SQUARE_MASK.trailing_zeros();
pub const TARGET_SQUARE_SHIFT: ShiftBits = TARGET_SQUARE_MASK.trailing_zeros();
pub const HALFMOVE_RESET_SHIFT: ShiftBits = HALFMOVE_RESET_MASK.trailing_zeros();
pub const PREVIOUS_HALFMOVE_SHIFT: ShiftBits = PREVIOUS_HALFMOVE_MASK.trailing_zeros();
pub const PREVIOUS_EN_PASSANT_SQUARE_SHIFT: ShiftBits = PREVIOUS_EN_PASSANT_SQUARE_MASK.trailing_zeros();
pub const NEXT_EN_PASSANT_SQUARE_SHIFT: ShiftBits = NEXT_EN_PASSANT_SQUARE_MASK.trailing_zeros();
pub const PROMOTION_PIECE_SHIFT: ShiftBits = PROMOTION_PIECE_MASK.trailing_zeros();

pub const NO_SQUARE: SquareShiftBits = 0;
pub const A8: SquareShiftBits = 0;
pub const B8: SquareShiftBits = 1;
pub const C8: SquareShiftBits = 2;
pub const D8: SquareShiftBits = 3;
pub const E8: SquareShiftBits = 4;
pub const F8: SquareShiftBits = 5;
pub const G8: SquareShiftBits = 6;
pub const H8: SquareShiftBits = 7;
pub const A7: SquareShiftBits = 8;
pub const B7: SquareShiftBits = 9;
pub const C7: SquareShiftBits = 10;
pub const D7: SquareShiftBits = 11;
pub const E7: SquareShiftBits = 12;
pub const F7: SquareShiftBits = 13;
pub const G7: SquareShiftBits = 14;
pub const H7: SquareShiftBits = 15;
pub const A6: SquareShiftBits = 16;
pub const B6: SquareShiftBits = 17;
pub const C6: SquareShiftBits = 18;
pub const D6: SquareShiftBits = 19;
pub const E6: SquareShiftBits = 20;
pub const F6: SquareShiftBits = 21;
pub const G6: SquareShiftBits = 22;
pub const H6: SquareShiftBits = 23;
pub const A5: SquareShiftBits = 24;
pub const B5: SquareShiftBits = 25;
pub const C5: SquareShiftBits = 26;
pub const D5: SquareShiftBits = 27;
pub const E5: SquareShiftBits = 28;
pub const F5: SquareShiftBits = 29;
pub const G5: SquareShiftBits = 30;
pub const H5: SquareShiftBits = 31;
pub const A4: SquareShiftBits = 32;
pub const B4: SquareShiftBits = 33;
pub const C4: SquareShiftBits = 34;
pub const D4: SquareShiftBits = 35;
pub const E4: SquareShiftBits = 36;
pub const F4: SquareShiftBits = 37;
pub const G4: SquareShiftBits = 38;
pub const H4: SquareShiftBits = 39;
pub const A3: SquareShiftBits = 40;
pub const B3: SquareShiftBits = 41;
pub const C3: SquareShiftBits = 42;
pub const D3: SquareShiftBits = 43;
pub const E3: SquareShiftBits = 44;
pub const F3: SquareShiftBits = 45;
pub const G3: SquareShiftBits = 46;
pub const H3: SquareShiftBits = 47;
pub const A2: SquareShiftBits = 48;
pub const B2: SquareShiftBits = 49;
pub const C2: SquareShiftBits = 50;
pub const D2: SquareShiftBits = 51;
pub const E2: SquareShiftBits = 52;
pub const F2: SquareShiftBits = 53;
pub const G2: SquareShiftBits = 54;
pub const H2: SquareShiftBits = 55;
pub const A1: SquareShiftBits = 56;
pub const B1: SquareShiftBits = 57;
pub const C1: SquareShiftBits = 58;
pub const D1: SquareShiftBits = 59;
pub const E1: SquareShiftBits = 60;
pub const F1: SquareShiftBits = 61;
pub const G1: SquareShiftBits = 62;
pub const H1: SquareShiftBits = 63;

pub const A8_MASK: SquareMaskBits = 1 << A8;
pub const B8_MASK: SquareMaskBits = 1 << B8;
pub const C8_MASK: SquareMaskBits = 1 << C8;
pub const D8_MASK: SquareMaskBits = 1 << D8;
pub const E8_MASK: SquareMaskBits = 1 << E8;
pub const F8_MASK: SquareMaskBits = 1 << F8;
pub const G8_MASK: SquareMaskBits = 1 << G8;
pub const H8_MASK: SquareMaskBits = 1 << H8;
pub const A7_MASK: SquareMaskBits = 1 << A7;
pub const B7_MASK: SquareMaskBits = 1 << B7;
pub const C7_MASK: SquareMaskBits = 1 << C7;
pub const D7_MASK: SquareMaskBits = 1 << D7;
pub const E7_MASK: SquareMaskBits = 1 << E7;
pub const F7_MASK: SquareMaskBits = 1 << F7;
pub const G7_MASK: SquareMaskBits = 1 << G7;
pub const H7_MASK: SquareMaskBits = 1 << H7;
pub const A6_MASK: SquareMaskBits = 1 << A6;
pub const B6_MASK: SquareMaskBits = 1 << B6;
pub const C6_MASK: SquareMaskBits = 1 << C6;
pub const D6_MASK: SquareMaskBits = 1 << D6;
pub const E6_MASK: SquareMaskBits = 1 << E6;
pub const F6_MASK: SquareMaskBits = 1 << F6;
pub const G6_MASK: SquareMaskBits = 1 << G6;
pub const H6_MASK: SquareMaskBits = 1 << H6;
pub const A5_MASK: SquareMaskBits = 1 << A5;
pub const B5_MASK: SquareMaskBits = 1 << B5;
pub const C5_MASK: SquareMaskBits = 1 << C5;
pub const D5_MASK: SquareMaskBits = 1 << D5;
pub const E5_MASK: SquareMaskBits = 1 << E5;
pub const F5_MASK: SquareMaskBits = 1 << F5;
pub const G5_MASK: SquareMaskBits = 1 << G5;
pub const H5_MASK: SquareMaskBits = 1 << H5;
pub const A4_MASK: SquareMaskBits = 1 << A4;
pub const B4_MASK: SquareMaskBits = 1 << B4;
pub const C4_MASK: SquareMaskBits = 1 << C4;
pub const D4_MASK: SquareMaskBits = 1 << D4;
pub const E4_MASK: SquareMaskBits = 1 << E4;
pub const F4_MASK: SquareMaskBits = 1 << F4;
pub const G4_MASK: SquareMaskBits = 1 << G4;
pub const H4_MASK: SquareMaskBits = 1 << H4;
pub const A3_MASK: SquareMaskBits = 1 << A3;
pub const B3_MASK: SquareMaskBits = 1 << B3;
pub const C3_MASK: SquareMaskBits = 1 << C3;
pub const D3_MASK: SquareMaskBits = 1 << D3;
pub const E3_MASK: SquareMaskBits = 1 << E3;
pub const F3_MASK: SquareMaskBits = 1 << F3;
pub const G3_MASK: SquareMaskBits = 1 << G3;
pub const H3_MASK: SquareMaskBits = 1 << H3;
pub const A2_MASK: SquareMaskBits = 1 << A2;
pub const B2_MASK: SquareMaskBits = 1 << B2;
pub const C2_MASK: SquareMaskBits = 1 << C2;
pub const D2_MASK: SquareMaskBits = 1 << D2;
pub const E2_MASK: SquareMaskBits = 1 << E2;
pub const F2_MASK: SquareMaskBits = 1 << F2;
pub const G2_MASK: SquareMaskBits = 1 << G2;
pub const H2_MASK: SquareMaskBits = 1 << H2;
pub const A1_MASK: SquareMaskBits = 1 << A1;
pub const B1_MASK: SquareMaskBits = 1 << B1;
pub const C1_MASK: SquareMaskBits = 1 << C1;
pub const D1_MASK: SquareMaskBits = 1 << D1;
pub const E1_MASK: SquareMaskBits = 1 << E1;
pub const F1_MASK: SquareMaskBits = 1 << F1;
pub const G1_MASK: SquareMaskBits = 1 << G1;
pub const H1_MASK: SquareMaskBits = 1 << H1;

pub const WHITE_QUEEN_SIDE_CASTLE_EMPTY_OCCUPANCY: OccupancyBits = B1_MASK | C1_MASK | D1_MASK;
pub const WHITE_KING_SIDE_CASTLE_EMPTY_OCCUPANCY: OccupancyBits = F1_MASK | G1_MASK;

pub const BLACK_QUEEN_SIDE_CASTLE_EMPTY_OCCUPANCY: OccupancyBits = B8_MASK | C8_MASK | D8_MASK;
pub const BLACK_KING_SIDE_CASTLE_EMPTY_OCCUPANCY: OccupancyBits = F8_MASK | G8_MASK;


pub const WHITE_QUEEN_SIDE_CASTLE_CHECK_OCCUPANCY: OccupancyBits = C1_MASK | D1_MASK | E1_MASK;
pub const WHITE_KING_SIDE_CASTLE_CHECK_OCCUPANCY: OccupancyBits = E1_MASK | F1_MASK | G1_MASK;

pub const BLACK_QUEEN_SIDE_CASTLE_CHECK_OCCUPANCY: OccupancyBits = C8_MASK | D8_MASK | E8_MASK;
pub const BLACK_KING_SIDE_CASTLE_CHECK_OCCUPANCY: OccupancyBits = E8_MASK | F8_MASK | G8_MASK;

pub const RANK_1_OCCUPANCY: OccupancyBits = A1_MASK | B1_MASK | C1_MASK | D1_MASK | E1_MASK | F1_MASK | G1_MASK | H1_MASK;
pub const RANK_2_OCCUPANCY: OccupancyBits = A2_MASK | B2_MASK | C2_MASK | D2_MASK | E2_MASK | F2_MASK | G2_MASK | H2_MASK;
pub const RANK_3_OCCUPANCY: OccupancyBits = A3_MASK | B3_MASK | C3_MASK | D3_MASK | E3_MASK | F3_MASK | G3_MASK | H3_MASK;
pub const RANK_4_OCCUPANCY: OccupancyBits = A4_MASK | B4_MASK | C4_MASK | D4_MASK | E4_MASK | F4_MASK | G4_MASK | H4_MASK;
pub const RANK_5_OCCUPANCY: OccupancyBits = A5_MASK | B5_MASK | C5_MASK | D5_MASK | E5_MASK | F5_MASK | G5_MASK | H5_MASK;
pub const RANK_6_OCCUPANCY: OccupancyBits = A6_MASK | B6_MASK | C6_MASK | D6_MASK | E6_MASK | F6_MASK | G6_MASK | H6_MASK;
pub const RANK_7_OCCUPANCY: OccupancyBits = A7_MASK | B7_MASK | C7_MASK | D7_MASK | E7_MASK | F7_MASK | G7_MASK | H7_MASK;
pub const RANK_8_OCCUPANCY: OccupancyBits = A8_MASK | B8_MASK | C8_MASK | D8_MASK | E8_MASK | F8_MASK | G8_MASK | H8_MASK;

pub const CASTLE_MOVE_TRUE_MASK: u64 = CASTLE_MOVE_MASK;
pub const CASTLE_MOVE_FALSE_MASK: u64 = 0;

pub const EN_PASSANT_ATTACK_TRUE_MASK: u64 = EN_PASSANT_ATTACK_MASK;
pub const EN_PASSANT_ATTACK_FALSE_MASK: u64 = 0;

// #[inline(always)]
pub fn square_mask_from_index(file_index: u32, rank_index: u32) -> SquareMaskBits {
    1 << square_shift_from_index(file_index, rank_index)
}

// #[inline(always)]
pub fn square_shift_from_index(file_index: u32, rank_index: u32) -> SquareShiftBits {
    to_square_index_from_indices(file_index as usize, rank_index as usize) as SquareShiftBits
}

// #[inline(always)]
pub fn square_mask_from_fen(fen: &str) -> SquareMaskBits {
    1 << square_shift_from_fen(fen)
}

// #[inline(always)]
pub fn square_shift_from_fen(fen: &str) -> SquareShiftBits {
    assert_eq!(fen.len(), 2, "Failed for {}", fen);
    let mut chars = fen.chars();

    let file_index = (chars.next().unwrap() as u8 - b'a') as u32;
    let rank_index = 8 - chars.next().unwrap().to_digit(10).unwrap();

    square_shift_from_index(file_index, rank_index)
}

pub fn fen_from_square_mask(square_mask: SquareMaskBits) -> String {
    fen_from_square_shift(square_mask.trailing_zeros())
}

pub fn fen_from_square_shift(square_shift: SquareShiftBits) -> String {
    assert!(square_shift < 64, "Should be valid square shift");

    let rank = 8 - (square_shift / 8);
    let file = square_shift % 8;

    let mut result = String::with_capacity(2);

    result.push(char::from_u32(file + 'a' as u32).unwrap());
    result.push(char::from_u32(rank + '0' as u32).unwrap());

    result
}

#[cfg(test)]
mod tests {
    use crate::board::constants::*;

    #[test]
    fn test_all_square_shift_from_fen() {
        test_square_shift_from_fen("a1", A1);
        test_square_shift_from_fen("b1", B1);
        test_square_shift_from_fen("c1", C1);
        test_square_shift_from_fen("d1", D1);
        test_square_shift_from_fen("e1", E1);
        test_square_shift_from_fen("f1", F1);
        test_square_shift_from_fen("g1", G1);
        test_square_shift_from_fen("h1", H1);
        test_square_shift_from_fen("a2", A2);
        test_square_shift_from_fen("b2", B2);
        test_square_shift_from_fen("c2", C2);
        test_square_shift_from_fen("d2", D2);
        test_square_shift_from_fen("e2", E2);
        test_square_shift_from_fen("f2", F2);
        test_square_shift_from_fen("g2", G2);
        test_square_shift_from_fen("h2", H2);
        test_square_shift_from_fen("a3", A3);
        test_square_shift_from_fen("b3", B3);
        test_square_shift_from_fen("c3", C3);
        test_square_shift_from_fen("d3", D3);
        test_square_shift_from_fen("e3", E3);
        test_square_shift_from_fen("f3", F3);
        test_square_shift_from_fen("g3", G3);
        test_square_shift_from_fen("h3", H3);
        test_square_shift_from_fen("a4", A4);
        test_square_shift_from_fen("b4", B4);
        test_square_shift_from_fen("c4", C4);
        test_square_shift_from_fen("d4", D4);
        test_square_shift_from_fen("e4", E4);
        test_square_shift_from_fen("f4", F4);
        test_square_shift_from_fen("g4", G4);
        test_square_shift_from_fen("h4", H4);
        test_square_shift_from_fen("a5", A5);
        test_square_shift_from_fen("b5", B5);
        test_square_shift_from_fen("c5", C5);
        test_square_shift_from_fen("d5", D5);
        test_square_shift_from_fen("e5", E5);
        test_square_shift_from_fen("f5", F5);
        test_square_shift_from_fen("g5", G5);
        test_square_shift_from_fen("h5", H5);
        test_square_shift_from_fen("a6", A6);
        test_square_shift_from_fen("b6", B6);
        test_square_shift_from_fen("c6", C6);
        test_square_shift_from_fen("d6", D6);
        test_square_shift_from_fen("e6", E6);
        test_square_shift_from_fen("f6", F6);
        test_square_shift_from_fen("g6", G6);
        test_square_shift_from_fen("h6", H6);
        test_square_shift_from_fen("a7", A7);
        test_square_shift_from_fen("b7", B7);
        test_square_shift_from_fen("c7", C7);
        test_square_shift_from_fen("d7", D7);
        test_square_shift_from_fen("e7", E7);
        test_square_shift_from_fen("f7", F7);
        test_square_shift_from_fen("g7", G7);
        test_square_shift_from_fen("h7", H7);
        test_square_shift_from_fen("a8", A8);
        test_square_shift_from_fen("b8", B8);
        test_square_shift_from_fen("c8", C8);
        test_square_shift_from_fen("d8", D8);
        test_square_shift_from_fen("e8", E8);
        test_square_shift_from_fen("f8", F8);
        test_square_shift_from_fen("g8", G8);
        test_square_shift_from_fen("h8", H8);
    }

    fn test_square_shift_from_fen(fen: &str, expected: SquareShiftBits) {
        assert_eq!(square_shift_from_fen(fen), expected, "fen {} should be {}", fen, expected)
    }

    #[test]
    fn test_all_fen_from_square_shift() {
        test_fen_from_square_shift(A1, "a1");
        test_fen_from_square_shift(B1, "b1");
        test_fen_from_square_shift(C1, "c1");
        test_fen_from_square_shift(D1, "d1");
        test_fen_from_square_shift(E1, "e1");
        test_fen_from_square_shift(F1, "f1");
        test_fen_from_square_shift(G1, "g1");
        test_fen_from_square_shift(H1, "h1");
        test_fen_from_square_shift(A2, "a2");
        test_fen_from_square_shift(B2, "b2");
        test_fen_from_square_shift(C2, "c2");
        test_fen_from_square_shift(D2, "d2");
        test_fen_from_square_shift(E2, "e2");
        test_fen_from_square_shift(F2, "f2");
        test_fen_from_square_shift(G2, "g2");
        test_fen_from_square_shift(H2, "h2");
        test_fen_from_square_shift(A3, "a3");
        test_fen_from_square_shift(B3, "b3");
        test_fen_from_square_shift(C3, "c3");
        test_fen_from_square_shift(D3, "d3");
        test_fen_from_square_shift(E3, "e3");
        test_fen_from_square_shift(F3, "f3");
        test_fen_from_square_shift(G3, "g3");
        test_fen_from_square_shift(H3, "h3");
        test_fen_from_square_shift(A4, "a4");
        test_fen_from_square_shift(B4, "b4");
        test_fen_from_square_shift(C4, "c4");
        test_fen_from_square_shift(D4, "d4");
        test_fen_from_square_shift(E4, "e4");
        test_fen_from_square_shift(F4, "f4");
        test_fen_from_square_shift(G4, "g4");
        test_fen_from_square_shift(H4, "h4");
        test_fen_from_square_shift(A5, "a5");
        test_fen_from_square_shift(B5, "b5");
        test_fen_from_square_shift(C5, "c5");
        test_fen_from_square_shift(D5, "d5");
        test_fen_from_square_shift(E5, "e5");
        test_fen_from_square_shift(F5, "f5");
        test_fen_from_square_shift(G5, "g5");
        test_fen_from_square_shift(H5, "h5");
        test_fen_from_square_shift(A6, "a6");
        test_fen_from_square_shift(B6, "b6");
        test_fen_from_square_shift(C6, "c6");
        test_fen_from_square_shift(D6, "d6");
        test_fen_from_square_shift(E6, "e6");
        test_fen_from_square_shift(F6, "f6");
        test_fen_from_square_shift(G6, "g6");
        test_fen_from_square_shift(H6, "h6");
        test_fen_from_square_shift(A7, "a7");
        test_fen_from_square_shift(B7, "b7");
        test_fen_from_square_shift(C7, "c7");
        test_fen_from_square_shift(D7, "d7");
        test_fen_from_square_shift(E7, "e7");
        test_fen_from_square_shift(F7, "f7");
        test_fen_from_square_shift(G7, "g7");
        test_fen_from_square_shift(H7, "h7");
        test_fen_from_square_shift(A8, "a8");
        test_fen_from_square_shift(B8, "b8");
        test_fen_from_square_shift(C8, "c8");
        test_fen_from_square_shift(D8, "d8");
        test_fen_from_square_shift(E8, "e8");
        test_fen_from_square_shift(F8, "f8");
        test_fen_from_square_shift(G8, "g8");
        test_fen_from_square_shift(H8, "h8");
    }

    fn test_fen_from_square_shift(square_shift: SquareShiftBits, expected: &str) {
        assert_eq!(fen_from_square_shift(square_shift), expected, "shift {} should be {}", square_shift, expected)
    }
}
