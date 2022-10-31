use std::char::from_digit;
use std::fmt::{Debug, Display, Formatter, write};

use marvk_chess_core::constants::color::Color;
use marvk_chess_core::constants::colored_piece::ColoredPiece;
use marvk_chess_core::constants::piece::Piece;
use marvk_chess_core::constants::square::Square;
use marvk_chess_core::constants::to_square_index_from_indices;
use marvk_chess_core::fen::Fen;

use crate::{highest_one_bit, move_to_san_reduced, occupancy_to_string, piece_to_string, square_to_string};
use crate::board::constants::*;
use crate::board::precalculated::magic::{BISHOP_MAGICS, Magics, ROOK_MAGICS};
use crate::board::precalculated::nonmagic::{BLACK_PAWN_NONMAGICS, KING_NONMAGICS, KNIGHT_NONMAGICS, Nonmagics, WHITE_PAWN_NONMAGICS};

pub mod constants;
mod precalculated;

#[derive(Eq, PartialEq, Hash)]
pub struct Move(pub u64);

impl Move {
    pub const NULL: Move = Move(0);

    #[inline(always)]
    pub fn get_piece_moved(&self) -> PieceBits { (self.0 & PIECE_MOVED_MASK) >> PIECE_MOVED_SHIFT }
    #[inline(always)]
    pub fn get_piece_attacked(&self) -> PieceBits { (self.0 & PIECE_ATTACKED_MASK) >> PIECE_ATTACKED_SHIFT }
    #[inline(always)]
    pub fn get_self_lost_king_side_castle(&self) -> u64 { (self.0 & SELF_LOST_KING_SIDE_CASTLE_MASK) >> SELF_LOST_KING_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_self_lost_queen_side_castle(&self) -> u64 { (self.0 & SELF_LOST_QUEEN_SIDE_CASTLE_MASK) >> SELF_LOST_QUEEN_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_opponent_lost_king_side_castle(&self) -> u64 { (self.0 & OPPONENT_LOST_KING_SIDE_CASTLE_MASK) >> OPPONENT_LOST_KING_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_opponent_lost_queen_side_castle(&self) -> u64 { (self.0 & OPPONENT_LOST_QUEEN_SIDE_CASTLE_MASK) >> OPPONENT_LOST_QUEEN_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_castle_move(&self) -> u64 { (self.0 & CASTLE_MOVE_MASK) >> CASTLE_MOVE_SHIFT }
    #[inline(always)]
    pub fn get_en_passant_attack(&self) -> u64 { (self.0 & EN_PASSANT_ATTACK_MASK) >> EN_PASSANT_ATTACK_SHIFT }
    #[inline(always)]
    pub fn get_source_square(&self) -> SquareShiftBits { ((self.0 & SOURCE_SQUARE_MASK) >> SOURCE_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_target_square(&self) -> SquareShiftBits { ((self.0 & TARGET_SQUARE_MASK) >> TARGET_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_halfmove_reset(&self) -> u64 { (self.0 & HALFMOVE_RESET_MASK) >> HALFMOVE_RESET_SHIFT }
    #[inline(always)]
    pub fn get_previous_halfmove(&self) -> u32 { ((self.0 & PREVIOUS_HALFMOVE_MASK) >> PREVIOUS_HALFMOVE_SHIFT) as u32 }
    #[inline(always)]
    pub fn get_previous_en_passant_square(&self) -> SquareShiftBits { ((self.0 & PREVIOUS_EN_PASSANT_SQUARE_MASK) >> PREVIOUS_EN_PASSANT_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_next_en_passant_square(&self) -> SquareShiftBits { ((self.0 & NEXT_EN_PASSANT_SQUARE_MASK) >> NEXT_EN_PASSANT_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_promotion_piece(&self) -> PieceBits { (self.0 & PROMOTION_PIECE_MASK) >> PROMOTION_PIECE_SHIFT }

    #[inline(always)]
    pub fn set_piece_moved(&mut self, value: PieceBits) { self.0 |= value << PIECE_MOVED_SHIFT }
    #[inline(always)]
    pub fn set_piece_attacked(&mut self, value: PieceBits) { self.0 |= value << PIECE_ATTACKED_SHIFT }
    #[inline(always)]
    pub fn set_self_lost_king_side_castle(&mut self) { self.0 |= SELF_LOST_KING_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_self_lost_queen_side_castle(&mut self) { self.0 |= SELF_LOST_QUEEN_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_opponent_lost_king_side_castle(&mut self) { self.0 |= OPPONENT_LOST_KING_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_opponent_lost_queen_side_castle(&mut self) { self.0 |= OPPONENT_LOST_QUEEN_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_castle_move(&mut self) { self.0 |= CASTLE_MOVE_MASK }
    #[inline(always)]
    pub fn set_en_passant_attack(&mut self) { self.0 |= EN_PASSANT_ATTACK_MASK }
    #[inline(always)]
    pub fn set_source_square(&mut self, value: SquareShiftBits) { self.0 |= (value as u64) << SOURCE_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_target_square(&mut self, value: SquareShiftBits) { self.0 |= (value as u64) << TARGET_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_halfmove_reset(&mut self, value: u64) { self.0 |= value << HALFMOVE_RESET_SHIFT }
    #[inline(always)]
    pub fn set_previous_halfmove(&mut self, value: u64) { self.0 |= value << PREVIOUS_HALFMOVE_SHIFT }
    #[inline(always)]
    pub fn set_previous_en_passant_square(&mut self, value: SquareShiftBits) { self.0 |= (value as u64) << PREVIOUS_EN_PASSANT_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_next_en_passant_square(&mut self, value: SquareShiftBits) { self.0 |= (value as u64) << NEXT_EN_PASSANT_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_promotion_piece(&mut self, value: PieceBits) { self.0 |= value << PROMOTION_PIECE_SHIFT }

    #[inline(always)]
    pub fn is_attack(&self) -> bool { self.get_piece_attacked() != 0 }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PlayerState {
    pub kings: OccupancyBits,
    pub queens: OccupancyBits,
    pub rooks: OccupancyBits,
    pub bishops: OccupancyBits,
    pub knights: OccupancyBits,
    pub pawns: OccupancyBits,
    pub queen_side_castle: bool,
    pub king_side_castle: bool,
}

impl PlayerState {
    pub const EMPTY: Self = Self { kings: 0, queens: 0, rooks: 0, bishops: 0, knights: 0, pawns: 0, queen_side_castle: false, king_side_castle: false };

    pub const fn new(kings: OccupancyBits, queens: OccupancyBits, rooks: OccupancyBits, bishops: OccupancyBits, knights: OccupancyBits, pawns: OccupancyBits, queen_side_castle: bool, king_side_castle: bool) -> Self {
        Self { kings, queens, rooks, bishops, knights, pawns, queen_side_castle, king_side_castle }
    }

    fn occupancy(&self) -> OccupancyBits {
        self.kings | self.queens | self.rooks | self.bishops | self.knights | self.pawns
    }

    fn unset_all(&mut self, occupancy: OccupancyBits) {
        let not_occupancy = !occupancy;
        self.kings &= not_occupancy;
        self.queens &= not_occupancy;
        self.rooks &= not_occupancy;
        self.bishops &= not_occupancy;
        self.knights &= not_occupancy;
        self.pawns &= not_occupancy;
    }

    fn get_piece_const_by_square_shift(&self, square_shift: SquareShiftBits) -> PieceBits {
        self.get_piece_const_by_square_mask(1_u64 << square_shift)
    }

    fn get_piece_const_by_square_mask(&self, square_mask: SquareMaskBits) -> PieceBits {
        if (self.pawns & square_mask) != 0 {
            PAWN
        } else if (self.knights & square_mask) != 0 {
            KNIGHT
        } else if (self.bishops & square_mask) != 0 {
            BISHOP
        } else if (self.rooks & square_mask) != 0 {
            ROOK
        } else if (self.queens & square_mask) != 0 {
            QUEEN
        } else if (self.kings & square_mask) != 0 {
            KING
        } else {
            NO_PIECE
        }
    }

    fn find_piece_struct_by_square_shift(&self, square: SquareShiftBits) -> Option<Piece> {
        self.find_piece_struct_by_square_mask(1 << square)
    }

    fn find_piece_struct_by_square_mask(&self, square: SquareMaskBits) -> Option<Piece> {
        if (self.pawns & square) != 0 {
            Some(Piece::PAWN)
        } else if (self.knights & square) != 0 {
            Some(Piece::KNIGHT)
        } else if (self.bishops & square) != 0 {
            Some(Piece::BISHOP)
        } else if (self.rooks & square) != 0 {
            Some(Piece::ROOK)
        } else if (self.queens & square) != 0 {
            Some(Piece::QUEEN)
        } else if (self.kings & square) != 0 {
            Some(Piece::KING)
        } else {
            None
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Bitboard {
    pub white: PlayerState,
    pub black: PlayerState,
    pub turn: ColorBits,
    pub en_passant_square_shift: SquareShiftBits,
    pub fullmove_clock: u32,
    pub halfmove_clock: u32,
}

// Instantiation
impl Bitboard {
    pub fn new(fen: &Fen) -> Self {
        let mut white = PlayerState::EMPTY.clone();
        let mut black = PlayerState::EMPTY.clone();

        fen.get_piece_placement().split("/").enumerate().for_each(|(rank_index, file)| {
            let mut file_index = 0;

            file.chars().into_iter().for_each(|c| {
                if c.is_ascii_digit() {
                    file_index += c.to_digit(10).unwrap()
                } else {
                    let board = if c.is_uppercase() { &mut white } else { &mut black };

                    let pieces = match c.to_ascii_lowercase() {
                        'p' => &mut board.pawns,
                        'n' => &mut board.knights,
                        'b' => &mut board.bishops,
                        'r' => &mut board.rooks,
                        'q' => &mut board.queens,
                        'k' => &mut board.kings,
                        _ => panic!(),
                    };

                    *pieces |= square_mask_from_index(file_index, rank_index as u32);

                    file_index += 1;
                }
            })
        });

        white.queen_side_castle = fen.get_castling_availability().contains('Q');
        white.king_side_castle = fen.get_castling_availability().contains('K');
        black.queen_side_castle = fen.get_castling_availability().contains('q');
        black.king_side_castle = fen.get_castling_availability().contains('k');

        let turn = match fen.get_active_color() {
            "b" => BLACK,
            "w" => WHITE,
            _ => panic!(),
        };
        let en_passant = if fen.get_en_passant_target_square() == "-" { NO_SQUARE } else { square_shift_from_fen(fen.get_en_passant_target_square()) };
        let fullmove_clock = fen.get_fullmove_clock().parse::<u32>().unwrap();
        let halfmove_clock = fen.get_halfmove_clock().parse::<u32>().unwrap();

        Self { white, black, turn, en_passant_square_shift: en_passant, fullmove_clock, halfmove_clock }
    }
}

// Move Generation
impl Bitboard {
    pub fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let (active, passive) = self.get_active_and_passive();

        let mut result: Vec<Move> = Vec::new();

        let active_occupancy = active.occupancy();
        let passive_occupancy = passive.occupancy();
        let full_occupancy = active_occupancy | passive_occupancy;

        self.sliding_attacks(&mut result, active.queens, active_occupancy, full_occupancy, &ROOK_MAGICS, QUEEN);
        self.sliding_attacks(&mut result, active.queens, active_occupancy, full_occupancy, &BISHOP_MAGICS, QUEEN);

        self.sliding_attacks(&mut result, active.bishops, active_occupancy, full_occupancy, &BISHOP_MAGICS, BISHOP);
        self.sliding_attacks(&mut result, active.rooks, active_occupancy, full_occupancy, &ROOK_MAGICS, ROOK);

        self.single_attacks(&mut result, active.knights, active_occupancy, &KNIGHT_NONMAGICS, KNIGHT);
        self.single_attacks(&mut result, active.kings, active_occupancy, &KING_NONMAGICS, KING);

        self.pawn_attacks(&mut result, active.pawns, active_occupancy, passive_occupancy);
        self.pawn_moves(&mut result, active.pawns, full_occupancy);

        self.castle_moves(&mut result, full_occupancy);

        result
    }


    fn sliding_attacks(
        &self,
        result: &mut Vec<Move>,
        mut piece_occupancy: OccupancyBits,
        active_occupancy: OccupancyBits,
        full_occupancy: OccupancyBits,
        magics: &Magics,
        piece: PieceBits,
    ) {
        while piece_occupancy != 0 {
            let source_square_mask = highest_one_bit(piece_occupancy);
            piece_occupancy &= !source_square_mask;
            let source_square_shift = source_square_mask.trailing_zeros();
            let attack_occupancy = magics.get_attacks(source_square_shift, full_occupancy) & !active_occupancy;

            self.generate_attacks(result, source_square_shift, attack_occupancy, piece);
        }
    }

    fn single_attacks(
        &self,
        result: &mut Vec<Move>,
        mut piece_occupancy: OccupancyBits,
        active_occupancy: OccupancyBits,
        nonmagics: &Nonmagics,
        piece: PieceBits,
    ) {
        while piece_occupancy != 0 {
            let source_square_mask = highest_one_bit(piece_occupancy);
            piece_occupancy &= !source_square_mask;
            let source_square_shift = source_square_mask.trailing_zeros();
            let attack_occupancy = nonmagics.get_attacks(source_square_shift) & !active_occupancy;

            self.generate_attacks(result, source_square_shift, attack_occupancy, piece);
        }
    }


    fn pawn_attacks(&self, result: &mut Vec<Move>, mut pawn_occupancy: OccupancyBits, active_occupancy: OccupancyBits, passive_occupancy: OccupancyBits) {
        let pawn_attacks: &Nonmagics = if self.is_white_turn() { &WHITE_PAWN_NONMAGICS } else { &BLACK_PAWN_NONMAGICS };

        while pawn_occupancy != 0 {
            let source_square_mask = highest_one_bit(pawn_occupancy);
            pawn_occupancy &= !source_square_mask;
            let source_square_shift = source_square_mask.trailing_zeros();

            let attack_occupancy =
                pawn_attacks.get_attacks(source_square_shift)
                    & (passive_occupancy | ((1 << self.en_passant_square_shift) & !(RANK_1_OCCUPANCY | RANK_8_OCCUPANCY)))
                    & !active_occupancy;

            self.generate_pawn_attacks(result, attack_occupancy, source_square_mask, source_square_shift)
        }
    }

    fn generate_pawn_attacks(&self, result: &mut Vec<Move>, mut attack_occupancy: OccupancyBits, source_square_mask: SquareMaskBits, source_square_shift: SquareShiftBits) {
        while attack_occupancy != 0 {
            let attack_square_mask: SquareMaskBits = highest_one_bit(attack_occupancy);
            attack_occupancy &= !attack_square_mask;
            let attack_square_shift = attack_square_mask.trailing_zeros();


            let is_white_turn = self.is_white_turn();

            if (is_white_turn && (attack_square_mask & RANK_8_OCCUPANCY) != 0) || (!is_white_turn && (attack_square_mask & RANK_1_OCCUPANCY) != 0) {
                self.generate_pawn_promotions(result, source_square_shift, attack_square_shift);
            } else {
                let is_en_passant = attack_square_shift == self.en_passant_square_shift;

                let x = self.make_move(source_square_shift, attack_square_shift, PAWN, false, is_en_passant, NO_PIECE, NO_SQUARE);
                result.push(x);
            }
        }
    }

    fn generate_pawn_promotions(&self, result: &mut Vec<Move>, source_square_shift: SquareShiftBits, target_square_shift: SquareShiftBits) {
        result.push(self.generate_pawn_promotion(source_square_shift, target_square_shift, QUEEN));
        result.push(self.generate_pawn_promotion(source_square_shift, target_square_shift, ROOK));
        result.push(self.generate_pawn_promotion(source_square_shift, target_square_shift, BISHOP));
        result.push(self.generate_pawn_promotion(source_square_shift, target_square_shift, KNIGHT));
    }

    fn generate_pawn_promotion(&self, source_square_shift: SquareShiftBits, attack_square_shift: SquareShiftBits, promote_to: PieceBits) -> Move {
        self.make_move(source_square_shift, attack_square_shift, PAWN, false, false, promote_to, NO_SQUARE)
    }

    fn pawn_moves(&self, result: &mut Vec<Move>, mut pawn_occupancy: OccupancyBits, full_occupancy: OccupancyBits) {
        while pawn_occupancy != 0 {
            let source_square_mask: SquareMaskBits = highest_one_bit(pawn_occupancy);
            pawn_occupancy &= !source_square_mask;
            let source_square_shift: SquareShiftBits = source_square_mask.trailing_zeros();

            let is_white_turn = self.is_white_turn();

            let single_move_target_mask;
            let promote_rank;

            if is_white_turn {
                single_move_target_mask = source_square_mask >> 8;
                promote_rank = RANK_8_OCCUPANCY;
            } else {
                single_move_target_mask = source_square_mask << 8;
                promote_rank = RANK_1_OCCUPANCY;
            }

            let single_move_target_shift = single_move_target_mask.trailing_zeros();

            if (single_move_target_mask & full_occupancy) == 0 {
                if (single_move_target_mask & promote_rank) != 0 {
                    self.generate_pawn_promotions(result, source_square_shift, single_move_target_shift);
                } else {
                    result.push(self.make_move(source_square_shift, single_move_target_shift, PAWN, false, false, NO_PIECE, NO_SQUARE));

                    let double_move_target_mask;
                    let double_move_source_rank;

                    if is_white_turn {
                        double_move_target_mask = single_move_target_mask >> 8;
                        double_move_source_rank = RANK_2_OCCUPANCY;
                    } else {
                        double_move_target_mask = single_move_target_mask << 8;
                        double_move_source_rank = RANK_7_OCCUPANCY;
                    }

                    if (source_square_mask & double_move_source_rank) != 0 && (double_move_target_mask & full_occupancy) == 0 {
                        result.push(self.make_move(source_square_shift, double_move_target_mask.trailing_zeros(), PAWN, false, false, NO_PIECE, single_move_target_shift));
                    }
                }
            }
        }
    }

    fn castle_moves(&self, result: &mut Vec<Move>, full_occupancy: OccupancyBits) {
        if self.is_white_turn() {
            if self.white.queen_side_castle
                && (full_occupancy & WHITE_QUEEN_SIDE_CASTLE_EMPTY_OCCUPANCY) == 0
                && !self._is_occupancy_in_check(WHITE, &self.black, full_occupancy, WHITE_QUEEN_SIDE_CASTLE_CHECK_OCCUPANCY) {
                result.push(self.make_castle_move(E1, C1));
            }

            if self.white.king_side_castle
                && (full_occupancy & WHITE_KING_SIDE_CASTLE_EMPTY_OCCUPANCY) == 0
                && !self._is_occupancy_in_check(WHITE, &self.black, full_occupancy, WHITE_KING_SIDE_CASTLE_CHECK_OCCUPANCY) {
                result.push(self.make_castle_move(E1, G1));
            }
        } else {
            if self.black.queen_side_castle
                && ((full_occupancy & BLACK_QUEEN_SIDE_CASTLE_EMPTY_OCCUPANCY) == 0)
                && !self._is_occupancy_in_check(BLACK, &self.white, full_occupancy, BLACK_QUEEN_SIDE_CASTLE_CHECK_OCCUPANCY) {
                result.push(self.make_castle_move(E8, C8));
            }

            if self.black.king_side_castle
                && (full_occupancy & BLACK_KING_SIDE_CASTLE_EMPTY_OCCUPANCY) == 0
                && !self._is_occupancy_in_check(BLACK, &self.white, full_occupancy, BLACK_KING_SIDE_CASTLE_CHECK_OCCUPANCY) {
                result.push(self.make_castle_move(E8, G8));
            }
        }
    }

    fn make_castle_move(&self, king_source_square_shift: SquareShiftBits, king_target_square_shift: SquareShiftBits) -> Move {
        self.make_move(king_source_square_shift, king_target_square_shift, KING, true, false, NO_PIECE, NO_SQUARE)
    }

    fn generate_attacks(
        &self,
        result: &mut Vec<Move>,
        source_square_shift: SquareShiftBits,
        mut attack_occupancy: OccupancyBits,
        piece: PieceBits,
    ) {
        while attack_occupancy != 0 {
            let target_square_mask = highest_one_bit(attack_occupancy);
            attack_occupancy &= !target_square_mask;
            let target_square_shift = target_square_mask.trailing_zeros();

            result.push(self.make_move(source_square_shift, target_square_shift, piece, false, false, NO_PIECE, NO_SQUARE));
        }
    }

    fn make_move(
        &self,
        source_square_shift: SquareShiftBits,
        target_square_shift: SquareShiftBits,
        piece_active: PieceBits,
        castle_move: bool,
        en_passant_move: bool,
        promote_to: PieceBits,
        en_passant_opportunity_square_shift: SquareShiftBits,
    ) -> Move {
        let attack_square_shift = if en_passant_move {
            if self.is_white_turn() {
                target_square_shift + 8
            } else {
                target_square_shift - 8
            }
        } else {
            target_square_shift
        };

        let piece_attacked = if self.is_white_turn() { self.black } else { self.white }.get_piece_const_by_square_shift(attack_square_shift);

        // if only_attack_moves {
        //     return Move::NULL;
        // }

        let mut mv = Move(0);

        if en_passant_move {
            mv.set_en_passant_attack();
        }

        mv.set_next_en_passant_square(en_passant_opportunity_square_shift);
        mv.set_piece_moved(piece_active);
        mv.set_piece_attacked(piece_attacked);

        mv.set_source_square(source_square_shift);
        mv.set_target_square(target_square_shift);

        if castle_move {
            mv.set_castle_move();
        }

        mv.set_previous_halfmove(self.halfmove_clock as u64);
        mv.set_previous_en_passant_square(self.en_passant_square_shift);

        mv.set_promotion_piece(promote_to);

        if self.is_white_turn() {
            if self.black.queen_side_castle && target_square_shift == A8 {
                mv.set_opponent_lost_queen_side_castle();
            } else if self.black.king_side_castle && target_square_shift == H8 {
                mv.set_opponent_lost_king_side_castle();
            }

            if self.white.queen_side_castle && (source_square_shift == A1 || source_square_shift == E1) {
                mv.set_self_lost_queen_side_castle();
            }

            if self.white.king_side_castle && (source_square_shift == H1 || source_square_shift == E1) {
                mv.set_self_lost_king_side_castle();
            }
        } else {
            if self.white.queen_side_castle && target_square_shift == A1 {
                mv.set_opponent_lost_queen_side_castle();
            } else if self.white.king_side_castle && target_square_shift == H1 {
                mv.set_opponent_lost_king_side_castle();
            }

            if self.black.queen_side_castle && (source_square_shift == A8 || source_square_shift == E8) {
                mv.set_self_lost_queen_side_castle();
            }

            if self.black.king_side_castle && (source_square_shift == H8 || source_square_shift == E8) {
                mv.set_self_lost_king_side_castle();
            }
        }

        mv
    }
}

// Make/Unmake move
impl Bitboard {
    pub fn make(&mut self, mv: Move) {
        let is_white_turn = self.is_white_turn();

        if !is_white_turn {
            self.fullmove_clock += 1;
        }

        if mv.get_halfmove_reset() != 0 {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        self.en_passant_square_shift = mv.get_next_en_passant_square();

        self.turn = if is_white_turn { BLACK } else { WHITE };

        // Swap active and passive because turn has already changed
        let (passive, active) = self.get_active_and_passive_mut();

        if mv.get_self_lost_king_side_castle() != 0 {
            active.king_side_castle = false;
        }

        if mv.get_self_lost_queen_side_castle() != 0 {
            active.queen_side_castle = false;
        }

        if mv.get_opponent_lost_king_side_castle() != 0 {
            passive.king_side_castle = false;
        }

        if mv.get_opponent_lost_queen_side_castle() != 0 {
            passive.queen_side_castle = false;
        }

        let source_square_shift = mv.get_source_square();
        let target_square_shift = mv.get_target_square();

        let source_square_mask: SquareMaskBits = 1_u64 << source_square_shift;
        let target_square_mask: SquareMaskBits = 1_u64 << target_square_shift;

        if mv.get_castle_move() != 0 {
            match target_square_shift {
                C1 => Self::make_castle(active, A1_MASK, E1_MASK, D1_MASK, C1_MASK),
                G1 => Self::make_castle(active, H1_MASK, E1_MASK, F1_MASK, G1_MASK),
                C8 => Self::make_castle(active, A8_MASK, E8_MASK, D8_MASK, C8_MASK),
                G8 => Self::make_castle(active, H8_MASK, E8_MASK, F8_MASK, G8_MASK),
                _ => {}
            };
        } else if mv.get_en_passant_attack() != 0 {
            active.pawns &= !source_square_mask;
            active.pawns |= target_square_mask;

            if is_white_turn {
                passive.unset_all(target_square_mask << 8)
            } else {
                passive.unset_all(target_square_mask >> 8)
            }
        } else {
            match mv.get_piece_moved() {
                KING => active.kings = (active.kings & !source_square_mask) | target_square_mask,
                QUEEN => active.queens = (active.queens & !source_square_mask) | target_square_mask,
                ROOK => active.rooks = (active.rooks & !source_square_mask) | target_square_mask,
                BISHOP => active.bishops = (active.bishops & !source_square_mask) | target_square_mask,
                KNIGHT => active.knights = (active.knights & !source_square_mask) | target_square_mask,
                PAWN => {
                    let promote = mv.get_promotion_piece();

                    if promote == NO_PIECE {
                        active.pawns = (active.pawns & !source_square_mask) | target_square_mask
                    } else {
                        active.pawns &= !source_square_mask;
                        match promote {
                            QUEEN => active.queens |= target_square_mask,
                            ROOK => active.rooks |= target_square_mask,
                            BISHOP => active.bishops |= target_square_mask,
                            KNIGHT => active.knights |= target_square_mask,
                            _ => panic!(),
                        }
                    }
                }
                _ => panic!(),
            }

            passive.unset_all(target_square_mask);
        }
    }


    pub fn unmake(&mut self, mv: Move) {
        let is_white_turn = self.is_white_turn();

        if is_white_turn {
            self.fullmove_clock -= 1;
        }
        self.halfmove_clock = mv.get_previous_halfmove();
        self.en_passant_square_shift = mv.get_previous_en_passant_square();
        self.turn = if is_white_turn { BLACK } else { WHITE };

        let (active, passive) = self.get_active_and_passive_mut();

        if mv.get_self_lost_king_side_castle() != 0 {
            active.king_side_castle = true;
        }

        if mv.get_self_lost_queen_side_castle() != 0 {
            active.queen_side_castle = true;
        }

        if mv.get_opponent_lost_king_side_castle() != 0 {
            passive.king_side_castle = true;
        }

        if mv.get_opponent_lost_queen_side_castle() != 0 {
            passive.queen_side_castle = true;
        }

        let source_square_shift = mv.get_source_square();
        let target_square_shift = mv.get_target_square();

        let source_square_mask: SquareMaskBits = 1_u64 << source_square_shift;
        let target_square_mask: SquareMaskBits = 1_u64 << target_square_shift;

        let piece_moved = mv.get_piece_moved();
        let piece_attacked = mv.get_piece_attacked();

        if mv.get_castle_move() != 0 {
            match target_square_shift {
                C1 => Self::unmake_castle(active, A1_MASK, E1_MASK, D1_MASK, C1_MASK),
                G1 => Self::unmake_castle(active, H1_MASK, E1_MASK, F1_MASK, G1_MASK),
                C8 => Self::unmake_castle(active, A8_MASK, E8_MASK, D8_MASK, C8_MASK),
                G8 => Self::unmake_castle(active, H8_MASK, E8_MASK, F8_MASK, G8_MASK),
                _ => {}
            };
        } else if mv.get_en_passant_attack() != 0 {
            active.pawns |= source_square_mask;
            active.pawns &= !target_square_mask;

            let en_passant_attack_target_mask = if is_white_turn {
                target_square_mask >> 8
            } else {
                target_square_mask << 8
            };

            match piece_attacked {
                KING => passive.kings |= en_passant_attack_target_mask,
                QUEEN => passive.queens |= en_passant_attack_target_mask,
                ROOK => passive.rooks |= en_passant_attack_target_mask,
                BISHOP => passive.bishops |= en_passant_attack_target_mask,
                KNIGHT => passive.knights |= en_passant_attack_target_mask,
                PAWN => passive.pawns |= en_passant_attack_target_mask,
                _ => panic!(),
            }
        } else {
            match piece_attacked {
                KING => passive.kings |= target_square_mask,
                QUEEN => passive.queens |= target_square_mask,
                ROOK => passive.rooks |= target_square_mask,
                BISHOP => passive.bishops |= target_square_mask,
                KNIGHT => passive.knights |= target_square_mask,
                PAWN => passive.pawns |= target_square_mask,
                _ => {}
            }

            match piece_moved {
                KING => active.kings |= source_square_mask,
                QUEEN => active.queens |= source_square_mask,
                ROOK => active.rooks |= source_square_mask,
                BISHOP => active.bishops |= source_square_mask,
                KNIGHT => active.knights |= source_square_mask,
                PAWN => active.pawns |= source_square_mask,
                _ => {}
            }

            active.unset_all(target_square_mask);
        }
    }

    fn make_castle(
        active: &mut PlayerState,
        rook_source_mask: SquareMaskBits,
        king_source_mask: SquareMaskBits,
        rook_target_mask: SquareMaskBits,
        king_target_mask: SquareMaskBits,
    ) {
        active.rooks &= !rook_source_mask;
        active.kings &= !king_source_mask;

        active.rooks |= rook_target_mask;
        active.kings |= king_target_mask;
    }

    fn unmake_castle(
        active: &mut PlayerState,
        rook_source_mask: SquareMaskBits,
        king_source_mask: SquareMaskBits,
        rook_target_mask: SquareMaskBits,
        king_target_mask: SquareMaskBits,
    ) {
        Self::make_castle(
            active,
            rook_target_mask,
            king_target_mask,
            rook_source_mask,
            king_source_mask,
        )
    }
}

// Validity Checks
impl Bitboard {
    pub fn is_valid(&self) -> bool {
        !self._is_in_check_by_bits(if self.is_white_turn() { BLACK } else { WHITE })
    }

    pub fn is_current_in_check(&self) -> bool {
        !self._is_in_check_by_bits(self.turn)
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        self._is_in_check_by_bits(color.index)
    }

    fn _is_in_check_by_bits(&self, color_bits: ColorBits) -> bool {
        let (active, passive) = if color_bits == WHITE {
            (&self.white, &self.black)
        } else {
            (&self.black, &self.white)
        };

        let full_occupancy = active.occupancy() | passive.occupancy();

        self._is_occupancy_in_check(color_bits, passive, full_occupancy, active.kings)
    }

    fn _is_occupancy_in_check(&self, color_bits: ColorBits, passive: &PlayerState, full_occupancy: OccupancyBits, mut king_occupancy: OccupancyBits) -> bool {
        while king_occupancy != 0 {
            let king = highest_one_bit(king_occupancy);
            king_occupancy &= !king;

            if self._is_square_in_check(color_bits, passive, king, full_occupancy) {
                return true;
            }
        }

        false
    }

    fn _is_square_in_check(&self, color_bits: ColorBits, passive: &PlayerState, king_square_mask: u64, full_occupancy: OccupancyBits) -> bool {
        let king_square_shift = king_square_mask.trailing_zeros();

        let rook_attacks = ROOK_MAGICS.get_attacks(king_square_shift, full_occupancy);

        if (rook_attacks & (passive.rooks | passive.queens)) != 0 {
            return true;
        }

        let bishop_attacks = BISHOP_MAGICS.get_attacks(king_square_shift, full_occupancy);

        if (bishop_attacks & (passive.bishops | passive.queens)) != 0 {
            return true;
        }

        let knight_attacks = KNIGHT_NONMAGICS.get_attacks(king_square_shift);

        if (knight_attacks & passive.knights) != 0 {
            return true;
        }

        let pawn_attacks = if color_bits == WHITE {
            WHITE_PAWN_NONMAGICS.get_attacks(king_square_shift)
        } else {
            BLACK_PAWN_NONMAGICS.get_attacks(king_square_shift)
        };

        if (pawn_attacks & passive.pawns) != 0 {
            return true;
        }

        let king_attacks = KING_NONMAGICS.get_attacks(king_square_shift);

        return (king_attacks & passive.kings) != 0;
    }
}

// Helpers
impl Bitboard {
    fn is_white_turn(&self) -> bool {
        self.turn == WHITE
    }

    fn get_active_and_passive(&self) -> (&PlayerState, &PlayerState) {
        if self.is_white_turn() {
            (&self.white, &self.black)
        } else {
            (&self.black, &self.white)
        }
    }

    fn get_active_and_passive_mut(&mut self) -> (&mut PlayerState, &mut PlayerState) {
        if self.is_white_turn() {
            (&mut self.white, &mut self.black)
        } else {
            (&mut self.black, &mut self.white)
        }
    }

    pub fn get_colored_piece(&self, square: Square) -> Option<ColoredPiece> {
        let maybe_white = self.white.find_piece_struct_by_square_mask(square.mask);
        let maybe_black = self.black.find_piece_struct_by_square_mask(square.mask);

        match (maybe_white, maybe_black) {
            (Some(piece), None) => Some(piece.as_white()),
            (None, Some(piece)) => Some(piece.as_black()),
            (None, None) => None,
            (Some(_), Some(_)) => panic!(),
        }
    }

    fn opposite_turn(&self) -> ColorBits {
        1 - self.turn
    }

    pub fn fen(&self) -> Fen {
        let mut result = String::new();

        for rank in 0..8 {
            let mut consecutive_empty = 0;
            for file in 0..8 {
                let square = Square::by_indices(file, rank).unwrap();
                let maybe_piece = self.get_colored_piece(square);
                match maybe_piece {
                    Some(piece) => {
                        if consecutive_empty > 0 {
                            result.push(from_digit(consecutive_empty, 10).unwrap());
                        }
                        consecutive_empty = 0;
                        result.push(piece.fen);
                    }
                    None => {
                        consecutive_empty += 1;
                    }
                };
            }
            if consecutive_empty > 0 {
                result.push(from_digit(consecutive_empty, 10).unwrap());
            }
            if rank < 7 {
                result.push('/');
            }
        }

        result.push(' ');
        result.push(if self.is_white_turn() { 'w' } else { 'b' });
        result.push(' ');

        let castle = [
            ('K', self.white.king_side_castle),
            ('Q', self.white.queen_side_castle),
            ('k', self.black.king_side_castle),
            ('q', self.black.queen_side_castle)
        ].iter().filter(|t| t.1).map(|t| t.0).collect::<String>();

        if castle.is_empty() {
            result.push('-');
        } else {
            result.push_str(&castle);
        };

        result.push(' ');

        if self.en_passant_square_shift != NO_SQUARE {
            result.push_str(&square_to_string(self.en_passant_square_shift));
        } else {
            result.push('-');
        }

        result.push(' ');
        result.push_str(&self.halfmove_clock.to_string());
        result.push(' ');
        result.push_str(&self.fullmove_clock.to_string());

        Fen::new(&result).unwrap()
    }

    pub fn perft(&mut self, depth: usize) -> Vec<(Move, u64)> {
        let mut result = Vec::new();

        for mv in self.generate_pseudo_legal_moves() {
            self.make(Move(mv.0));

            if self.is_valid() {
                result.push((Move(mv.0), self._perft(depth - 1)));
            }

            self.unmake(mv);
        }

        result
    }

    fn _perft(&mut self, depth: usize) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut count = 0;

        for mv in self.generate_pseudo_legal_moves() {
            self.make(Move(mv.0));

            if self.is_valid() {
                count += self._perft(depth - 1);
            }

            self.unmake(mv);
        }

        count
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Move({}) {{ piece_moved = {}, piece_attacked = {}, self_lost_king_side_castle = {}, self_lost_queen_side_castle = {}, opponent_lost_king_side_castle = {}, opponent_lost_queen_side_castle = {}, castle_move = {}, en_passant_attack = {}, source_square = {}, target_square = {}, halfmove_reset = {}, previous_halfmove = {}, previous_en_passant_square = {}, next_en_passant_square = {}, promotion_piece = {}}}",
            move_to_san_reduced(&self),
            piece_to_string(self.get_piece_moved()),
            piece_to_string(self.get_piece_attacked()),
            self.get_self_lost_king_side_castle() != 0,
            self.get_self_lost_queen_side_castle() != 0,
            self.get_opponent_lost_king_side_castle() != 0,
            self.get_opponent_lost_queen_side_castle() != 0,
            self.get_castle_move() != 0,
            self.get_en_passant_attack() != 0,
            square_to_string(self.get_source_square()),
            square_to_string(self.get_target_square()),
            self.get_halfmove_reset() != 0,
            self.get_previous_halfmove(),
            square_to_string(self.get_previous_en_passant_square()),
            square_to_string(self.get_next_en_passant_square()),
            piece_to_string(self.get_promotion_piece()),
        )
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();

        for rank in 0..8 {
            board.push(char::from_digit(8 - rank, 10).unwrap());
            for file in 0..8 {
                let square_mask = square_mask_from_index(file, rank);
                let white_piece = self.white.find_piece_struct_by_square_mask(square_mask);
                let black_piece = self.black.find_piece_struct_by_square_mask(square_mask);

                let char = if let Some(white_piece) = white_piece {
                    if black_piece.is_some() {
                        panic!("two pieces on the same square")
                    }
                    white_piece.as_white().fen
                } else if let Some(black_piece) = black_piece {
                    black_piece.as_black().fen
                } else {
                    ' '
                };

                board.push_str(&format!(" {} ", char));

                if file < 7 {
                    board.push('│');
                }
            }
            board.push('║');
            if rank < 7 {
                board.push_str(&format!("\n╟{0}┼{0}┼{0}┼{0}┼{0}┼{0}┼{0}┼{0}╢\n", "───"));
            }
        }

        let mut other = String::new();


        fn format_information<T: Display>(key: &str, value: T) -> String {
            format!("║ {: <30}║\n", format!("{: >14}: {}", key, value))
        }

        other.push_str(&format_information("fullmove clock", self.fullmove_clock));
        other.push_str(&format_information("halfmove clock", self.halfmove_clock));
        other.push_str(&format_information("turn", if self.is_white_turn() { "white" } else { "black" }));
        other.push_str(&format_information("en passant", if self.en_passant_square_shift == NO_SQUARE { "none".to_string() } else { fen_from_square_shift(self.en_passant_square_shift) }));

        write!(
            f,
            "╔{0}╤{0}╤{0}╤{0}╤{0}╤{0}╤{0}╤{0}╗\n{1}\n╠═A═╧═B═╧═C═╧═D═╧═E═╧═F═╧═G═╧═H═╣\n{2}╚{0}═{0}═{0}═{0}═{0}═{0}═{0}═{0}╝",
            "═══",
            board,
            other
        )
    }
}

#[cfg(test)]
mod tests {
    use marvk_chess_core::fen::Fen;

    use crate::board::{Bitboard, highest_one_bit, Move};

    #[test]
    fn test_fen() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
        ];

        for x in fens {
            let expected = Fen::new(x).unwrap();
            let actual = Bitboard::new(&expected).fen();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test2() {
        println!("{:b}", highest_one_bit(0b1));
        println!("{:b}", highest_one_bit(0b11));
        println!("{:b}", highest_one_bit(0b111));
        println!("{:b}", highest_one_bit(0b1111));
    }

    #[test]
    fn test() {
        let bitboard = Bitboard::new(&Fen::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap());
        println!("{:?}", bitboard.white);
        println!("---");
        println!("{:?}", bitboard.black);
        println!("---");
        println!("{}", bitboard);
    }

    #[test]
    fn gen_test1() {
        let mut bitboard = Bitboard::new(&Fen::new("4k3/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap());

        let vec = bitboard.generate_pseudo_legal_moves();

        for x in vec {
            let y = Move(x.0);
            bitboard.make(x);
            println!("{}", bitboard);
            bitboard.unmake(y);
        }

        println!("{}", bitboard);
    }
}

