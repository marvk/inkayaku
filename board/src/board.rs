use std::char::from_digit;
use std::fmt::{Debug, Display, Formatter};

use marvk_chess_core::constants::color::Color;
use marvk_chess_core::constants::colored_piece::ColoredPiece;
use marvk_chess_core::constants::file::File;
use marvk_chess_core::constants::piece::Piece;
use marvk_chess_core::constants::square::Square;
use marvk_chess_core::fen::{Fen, FEN_STARTPOS};

use crate::{mask_and_shift_from_lowest_one_bit, opposite_color, piece_to_string, square_to_string};
use crate::board::constants::*;
use crate::board::MoveFromUciError::{MoveDoesNotExist, MoveIsNotValid};
use crate::board::precalculated::magic::{BISHOP_MAGICS, Magics, ROOK_MAGICS};
use crate::board::precalculated::nonmagic::{BLACK_PAWN_NONMAGICS, KING_NONMAGICS, KNIGHT_NONMAGICS, Nonmagics, WHITE_PAWN_NONMAGICS};
use crate::board::zobrist::Zobrist;

pub mod constants;
mod precalculated;
mod zobrist;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct Move {
    pub bits: u64,
    pub mvvlva: i32,
}

impl Move {
    #[inline(always)]
    pub fn get_piece_moved(&self) -> PieceBits { (self.bits & PIECE_MOVED_MASK) >> PIECE_MOVED_SHIFT }
    #[inline(always)]
    pub fn get_piece_attacked(&self) -> PieceBits { (self.bits & PIECE_ATTACKED_MASK) >> PIECE_ATTACKED_SHIFT }
    #[inline(always)]
    pub fn get_self_lost_king_side_castle(&self) -> u64 { (self.bits & SELF_LOST_KING_SIDE_CASTLE_MASK) >> SELF_LOST_KING_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_self_lost_queen_side_castle(&self) -> u64 { (self.bits & SELF_LOST_QUEEN_SIDE_CASTLE_MASK) >> SELF_LOST_QUEEN_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_opponent_lost_king_side_castle(&self) -> u64 { (self.bits & OPPONENT_LOST_KING_SIDE_CASTLE_MASK) >> OPPONENT_LOST_KING_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_opponent_lost_queen_side_castle(&self) -> u64 { (self.bits & OPPONENT_LOST_QUEEN_SIDE_CASTLE_MASK) >> OPPONENT_LOST_QUEEN_SIDE_CASTLE_SHIFT }
    #[inline(always)]
    pub fn get_castle_move(&self) -> u64 { (self.bits & CASTLE_MOVE_MASK) >> CASTLE_MOVE_SHIFT }
    #[inline(always)]
    pub fn get_en_passant_attack(&self) -> u64 { (self.bits & EN_PASSANT_ATTACK_MASK) >> EN_PASSANT_ATTACK_SHIFT }
    #[inline(always)]
    pub fn get_source_square(&self) -> SquareShiftBits { ((self.bits & SOURCE_SQUARE_MASK) >> SOURCE_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_target_square(&self) -> SquareShiftBits { ((self.bits & TARGET_SQUARE_MASK) >> TARGET_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_halfmove_reset(&self) -> u64 { (self.bits & HALFMOVE_RESET_MASK) >> HALFMOVE_RESET_SHIFT }
    #[inline(always)]
    pub fn get_previous_halfmove(&self) -> u32 { ((self.bits & PREVIOUS_HALFMOVE_MASK) >> PREVIOUS_HALFMOVE_SHIFT) as u32 }
    #[inline(always)]
    pub fn get_previous_en_passant_square(&self) -> SquareShiftBits { ((self.bits & PREVIOUS_EN_PASSANT_SQUARE_MASK) >> PREVIOUS_EN_PASSANT_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_next_en_passant_square(&self) -> SquareShiftBits { ((self.bits & NEXT_EN_PASSANT_SQUARE_MASK) >> NEXT_EN_PASSANT_SQUARE_SHIFT) as SquareShiftBits }
    #[inline(always)]
    pub fn get_promotion_piece(&self) -> PieceBits { (self.bits & PROMOTION_PIECE_MASK) >> PROMOTION_PIECE_SHIFT }
    #[inline(always)]
    pub fn get_side_to_move(&self) -> ColorBits { ((self.bits & SIDE_TO_MOVE_MASK) >> SIDE_TO_MOVE_SHIFT) as ColorBits }

    #[inline(always)]
    pub fn set_piece_moved(&mut self, value: PieceBits) { self.bits |= value << PIECE_MOVED_SHIFT }
    #[inline(always)]
    pub fn set_piece_attacked(&mut self, value: PieceBits) { self.bits |= value << PIECE_ATTACKED_SHIFT }
    #[inline(always)]
    pub fn set_self_lost_king_side_castle(&mut self) { self.bits |= SELF_LOST_KING_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_self_lost_queen_side_castle(&mut self) { self.bits |= SELF_LOST_QUEEN_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_opponent_lost_king_side_castle(&mut self) { self.bits |= OPPONENT_LOST_KING_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_opponent_lost_queen_side_castle(&mut self) { self.bits |= OPPONENT_LOST_QUEEN_SIDE_CASTLE_MASK }
    #[inline(always)]
    pub fn set_castle_move(&mut self, value: u64) { self.bits |= value }
    #[inline(always)]
    pub fn set_en_passant_attack(&mut self, value: u64) { self.bits |= value }
    #[inline(always)]
    pub fn set_source_square(&mut self, value: SquareShiftBits) { self.bits |= (value as u64) << SOURCE_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_target_square(&mut self, value: SquareShiftBits) { self.bits |= (value as u64) << TARGET_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_halfmove_reset(&mut self) { self.bits |= HALFMOVE_RESET_MASK }
    #[inline(always)]
    pub fn set_previous_halfmove(&mut self, value: u32) { self.bits |= (value << PREVIOUS_HALFMOVE_SHIFT) as u64 }
    #[inline(always)]
    pub fn set_previous_en_passant_square(&mut self, value: SquareShiftBits) { self.bits |= (value as u64) << PREVIOUS_EN_PASSANT_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_next_en_passant_square(&mut self, value: SquareShiftBits) { self.bits |= (value as u64) << NEXT_EN_PASSANT_SQUARE_SHIFT }
    #[inline(always)]
    pub fn set_promotion_piece(&mut self, value: PieceBits) { self.bits |= value << PROMOTION_PIECE_SHIFT }
    #[inline(always)]
    pub fn set_side_to_move(&mut self, value: ColorBits) { self.bits |= (value as u64) << SIDE_TO_MOVE_SHIFT }

    #[inline(always)]
    pub fn is_self_lost_king_side_castle(&self) -> bool { self.get_self_lost_king_side_castle() != 0 }
    #[inline(always)]
    pub fn is_self_lost_queen_side_castle(&self) -> bool { self.get_self_lost_queen_side_castle() != 0 }
    #[inline(always)]
    pub fn is_opponent_lost_king_side_castle(&self) -> bool { self.get_opponent_lost_king_side_castle() != 0 }
    #[inline(always)]
    pub fn is_opponent_lost_queen_side_castle(&self) -> bool { self.get_opponent_lost_queen_side_castle() != 0 }
    #[inline(always)]
    pub fn is_en_passant_attack(&self) -> bool { self.get_en_passant_attack() != 0 }
    #[inline(always)]
    pub fn is_castle_move(&self) -> bool { self.get_castle_move() != 0 }
    #[inline(always)]
    pub fn is_halfmove_reset(&self) -> bool { self.get_halfmove_reset() != 0 }
    #[inline(always)]
    pub fn is_attack(&self) -> bool { self.get_piece_attacked() != NO_PIECE }
    #[inline(always)]
    pub fn is_promotion(&self) -> bool { self.get_promotion_piece() != NO_PIECE }

    pub fn to_uci_string(&self) -> String {
        format!("{}{}{}", square_to_string(self.get_source_square()), square_to_string(self.get_target_square()), piece_to_string(self.get_promotion_piece()))
    }

    pub fn to_pgn_string(&self, board: &mut Bitboard) -> Result<String, MoveFromUciError> {
        board.uci_to_pgn(&self.to_uci_string())
    }

    pub fn structs(&self) -> (Square, Square, Option<Piece>) {
        (Square::by_index(self.get_source_square() as usize).unwrap(),
         Square::by_index(self.get_target_square() as usize).unwrap(),
         Piece::by_index(self.get_promotion_piece() as usize))
    }
}

pub struct MoveStructs {
    pub from_square: Square,
    pub to_square: Square,
    pub from_piece: Piece,
    pub to_piece: Option<Piece>,
    pub promote_to: Option<Piece>,
}

impl From<Move> for MoveStructs {
    fn from(mv: Move) -> Self {
        MoveStructs {
            from_square: Square::by_index(mv.get_source_square() as usize).unwrap(),
            to_square: Square::by_index(mv.get_target_square() as usize).unwrap(),
            from_piece: Piece::by_index(mv.get_piece_moved() as usize).unwrap(),
            to_piece: Piece::by_index(mv.get_piece_attacked() as usize),
            promote_to: Piece::by_index(mv.get_promotion_piece() as usize),
        }
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.to_uci_string(),
        )
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum MoveFromUciError {
    MoveDoesNotExist(String),
    MoveIsNotValid(Move),
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub struct PlayerState {
    occupancy: [OccupancyBits; 7],
    pub queen_side_castle: bool,
    pub king_side_castle: bool,
}

impl PlayerState {
    fn full_occupancy(&self) -> OccupancyBits {
        self.kings() | self.queens() | self.rooks() | self.bishops() | self.knights() | self.pawns()
    }

    #[inline(always)]
    fn occupancy_ref(&mut self, piece: PieceBits) -> &mut OccupancyBits { &mut self.occupancy[piece as usize] }
    #[inline(always)]
    fn kings_ref(&mut self) -> &mut OccupancyBits { &mut self.occupancy[KING as usize] }
    #[inline(always)]
    fn queens_ref(&mut self) -> &mut OccupancyBits { &mut self.occupancy[QUEEN as usize] }
    #[inline(always)]
    fn rooks_ref(&mut self) -> &mut OccupancyBits { &mut self.occupancy[ROOK as usize] }
    #[inline(always)]
    fn bishops_ref(&mut self) -> &mut OccupancyBits { &mut self.occupancy[BISHOP as usize] }
    #[inline(always)]
    fn knights_ref(&mut self) -> &mut OccupancyBits { &mut self.occupancy[KNIGHT as usize] }
    #[inline(always)]
    fn pawns_ref(&mut self) -> &mut OccupancyBits { &mut self.occupancy[PAWN as usize] }

    #[inline(always)]
    fn occupancy(&self, piece: PieceBits) -> OccupancyBits { self.occupancy[piece as usize] }
    #[inline(always)]
    pub fn kings(&self) -> OccupancyBits { self.occupancy[KING as usize] }
    #[inline(always)]
    pub fn queens(&self) -> OccupancyBits { self.occupancy[QUEEN as usize] }
    #[inline(always)]
    pub fn rooks(&self) -> OccupancyBits { self.occupancy[ROOK as usize] }
    #[inline(always)]
    pub fn bishops(&self) -> OccupancyBits { self.occupancy[BISHOP as usize] }
    #[inline(always)]
    pub fn knights(&self) -> OccupancyBits { self.occupancy[KNIGHT as usize] }
    #[inline(always)]
    pub fn pawns(&self) -> OccupancyBits { self.occupancy[PAWN as usize] }

    fn get_piece_const_by_square_shift(&self, square_shift: SquareShiftBits) -> PieceBits {
        self.get_piece_const_by_square_mask(1_u64 << square_shift)
    }

    fn get_piece_const_by_square_mask(&self, square_mask: SquareMaskBits) -> PieceBits {
        if (self.pawns() & square_mask) != 0 {
            PAWN
        } else if (self.knights() & square_mask) != 0 {
            KNIGHT
        } else if (self.bishops() & square_mask) != 0 {
            BISHOP
        } else if (self.rooks() & square_mask) != 0 {
            ROOK
        } else if (self.queens() & square_mask) != 0 {
            QUEEN
        } else if (self.kings() & square_mask) != 0 {
            KING
        } else {
            NO_PIECE
        }
    }

    fn find_piece_struct_by_square_shift(&self, square: SquareShiftBits) -> Option<Piece> {
        self.find_piece_struct_by_square_mask(1 << square)
    }

    fn find_piece_struct_by_square_mask(&self, square: SquareMaskBits) -> Option<Piece> {
        Piece::by_index(self.get_piece_const_by_square_mask(square) as usize)
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

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard::new(&FEN_STARTPOS.clone())
    }
}

// Instantiation
impl Bitboard {
    pub fn new(fen: &Fen) -> Self {
        let mut white = PlayerState::default();
        let mut black = PlayerState::default();

        fen.get_piece_placement().split('/').enumerate().for_each(|(rank_index, file)| {
            let mut file_index = 0;

            file.chars().into_iter().for_each(|c| {
                if c.is_ascii_digit() {
                    file_index += c.to_digit(10).unwrap()
                } else {
                    let board = if c.is_uppercase() { &mut white } else { &mut black };

                    let pieces = match c.to_ascii_lowercase() {
                        'p' => board.pawns_ref(),
                        'n' => board.knights_ref(),
                        'b' => board.bishops_ref(),
                        'r' => board.rooks_ref(),
                        'q' => board.queens_ref(),
                        'k' => board.kings_ref(),
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
        let en_passant_square_shift = if fen.get_en_passant_target_square() == "-" { NO_SQUARE } else { square_shift_from_fen(fen.get_en_passant_target_square()) };
        let fullmove_clock = fen.get_fullmove_clock().parse::<u32>().unwrap();
        let halfmove_clock = fen.get_halfmove_clock().parse::<u32>().unwrap();

        Self { white, black, turn, en_passant_square_shift, fullmove_clock, halfmove_clock }
    }
}

// Move Generation
impl Bitboard {
    pub fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut buffer = Vec::new();
        self.generate_pseudo_legal_moves_with_buffer(&mut buffer);
        buffer
    }

    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        self.generate_pseudo_legal_moves()
            .into_iter()
            .filter(|&mv| self.is_move_legal(mv))
            .collect()
    }

    pub fn generate_pseudo_legal_non_quiescent_moves(&self) -> Vec<Move> {
        let mut buffer = Vec::new();
        self.generate_pseudo_legal_non_quiescent_moves_with_buffer(&mut buffer);
        buffer
    }

    pub fn generate_pseudo_legal_moves_with_buffer(&self, result: &mut Vec<Move>) {
        let (active, passive) = self.get_active_and_passive();

        let active_occupancy = active.full_occupancy();
        let passive_occupancy = passive.full_occupancy();
        let full_occupancy = active_occupancy | passive_occupancy;

        self.sliding_moves(result, false, active.queens(), active_occupancy, full_occupancy, &ROOK_MAGICS, QUEEN);
        self.sliding_moves(result, false, active.queens(), active_occupancy, full_occupancy, &BISHOP_MAGICS, QUEEN);

        self.sliding_moves(result, false, active.bishops(), active_occupancy, full_occupancy, &BISHOP_MAGICS, BISHOP);
        self.sliding_moves(result, false, active.rooks(), active_occupancy, full_occupancy, &ROOK_MAGICS, ROOK);

        self.single_moves(result, false, active.knights(), active_occupancy, &KNIGHT_NONMAGICS, KNIGHT);
        self.single_moves(result, false, active.kings(), active_occupancy, &KING_NONMAGICS, KING);

        self.pawn_attacks(result, active.pawns(), active_occupancy, passive_occupancy);
        self.pawn_moves(result, false, active.pawns(), full_occupancy);

        self.castle_moves(result, full_occupancy);
    }

    pub fn generate_pseudo_legal_non_quiescent_moves_with_buffer(&self, result: &mut Vec<Move>) {
        let (active, passive) = self.get_active_and_passive();

        let active_occupancy = active.full_occupancy();
        let passive_occupancy = passive.full_occupancy();
        let full_occupancy = active_occupancy | passive_occupancy;

        self.sliding_moves(result, true, active.queens(), active_occupancy, full_occupancy, &ROOK_MAGICS, QUEEN);
        self.sliding_moves(result, true, active.queens(), active_occupancy, full_occupancy, &BISHOP_MAGICS, QUEEN);

        self.sliding_moves(result, true, active.bishops(), active_occupancy, full_occupancy, &BISHOP_MAGICS, BISHOP);
        self.sliding_moves(result, true, active.rooks(), active_occupancy, full_occupancy, &ROOK_MAGICS, ROOK);

        self.single_moves(result, true, active.knights(), active_occupancy, &KNIGHT_NONMAGICS, KNIGHT);
        self.single_moves(result, true, active.kings(), active_occupancy, &KING_NONMAGICS, KING);

        self.pawn_attacks(result, active.pawns(), active_occupancy, passive_occupancy);
        self.pawn_moves(result, true, active.pawns(), full_occupancy);
    }


    fn sliding_moves(
        &self,
        result: &mut Vec<Move>,
        non_quiescent_only: bool,
        mut piece_occupancy: OccupancyBits,
        active_occupancy: OccupancyBits,
        full_occupancy: OccupancyBits,
        magics: &Magics,
        piece: PieceBits,
    ) {
        while piece_occupancy != 0 {
            let (source_square_mask, source_square_shift) = mask_and_shift_from_lowest_one_bit(piece_occupancy);
            piece_occupancy &= !source_square_mask;

            let attack_occupancy = magics.get_attacks(source_square_shift, full_occupancy) & !active_occupancy;

            self.generate_attacks(result, non_quiescent_only, source_square_shift, attack_occupancy, piece);
        }
    }

    fn single_moves(
        &self,
        result: &mut Vec<Move>,
        non_quiescent_only: bool,
        mut piece_occupancy: OccupancyBits,
        active_occupancy: OccupancyBits,
        nonmagics: &Nonmagics,
        piece: PieceBits,
    ) {
        while piece_occupancy != 0 {
            let (source_square_mask, source_square_shift) = mask_and_shift_from_lowest_one_bit(piece_occupancy);
            piece_occupancy &= !source_square_mask;

            let attack_occupancy = nonmagics.get_attacks(source_square_shift) & !active_occupancy;
            self.generate_attacks(result, non_quiescent_only, source_square_shift, attack_occupancy, piece);
        }
    }


    fn pawn_attacks(&self, result: &mut Vec<Move>, mut pawn_occupancy: OccupancyBits, active_occupancy: OccupancyBits, passive_occupancy: OccupancyBits) {
        let pawn_attacks: &Nonmagics = if self.is_white_turn() { &WHITE_PAWN_NONMAGICS } else { &BLACK_PAWN_NONMAGICS };

        while pawn_occupancy != 0 {
            let (source_square_mask, source_square_shift) = mask_and_shift_from_lowest_one_bit(pawn_occupancy);
            pawn_occupancy &= !source_square_mask;

            let attack_occupancy =
                pawn_attacks.get_attacks(source_square_shift)
                    & (passive_occupancy | ((1 << self.en_passant_square_shift) & !(RANK_1_OCCUPANCY | RANK_8_OCCUPANCY)))
                    & !active_occupancy;
            self.generate_pawn_attacks(result, attack_occupancy, source_square_shift)
        }
    }

    fn generate_pawn_attacks(&self, result: &mut Vec<Move>, mut attack_occupancy: OccupancyBits, source_square_shift: SquareShiftBits) {
        while attack_occupancy != 0 {
            let (attack_square_mask, attack_square_shift) = mask_and_shift_from_lowest_one_bit(attack_occupancy);
            attack_occupancy &= !attack_square_mask;

            if (attack_square_mask & RANK_8_OCCUPANCY) != 0 || (attack_square_mask & RANK_1_OCCUPANCY) != 0 {
                self.generate_pawn_promotions(result, source_square_shift, attack_square_shift);
            } else {
                let is_en_passant = attack_square_shift == self.en_passant_square_shift;

                self.make_move(
                    result,
                    false,
                    source_square_shift,
                    attack_square_shift,
                    PAWN,
                    CASTLE_MOVE_FALSE_MASK,
                    if is_en_passant { EN_PASSANT_ATTACK_TRUE_MASK } else { EN_PASSANT_ATTACK_FALSE_MASK },
                    NO_PIECE,
                    NO_SQUARE,
                );
            }
        }
    }

    fn generate_pawn_promotions(&self, result: &mut Vec<Move>, source_square_shift: SquareShiftBits, target_square_shift: SquareShiftBits) {
        self.generate_pawn_promotion(result, source_square_shift, target_square_shift, QUEEN);
        self.generate_pawn_promotion(result, source_square_shift, target_square_shift, ROOK);
        self.generate_pawn_promotion(result, source_square_shift, target_square_shift, BISHOP);
        self.generate_pawn_promotion(result, source_square_shift, target_square_shift, KNIGHT);
    }

    fn generate_pawn_promotion(&self, result: &mut Vec<Move>, source_square_shift: SquareShiftBits, attack_square_shift: SquareShiftBits, promote_to: PieceBits) {
        self.make_move(
            result,
            false,
            source_square_shift,
            attack_square_shift,
            PAWN,
            CASTLE_MOVE_FALSE_MASK,
            EN_PASSANT_ATTACK_FALSE_MASK,
            promote_to,
            NO_SQUARE,
        );
    }

    fn pawn_moves(&self, result: &mut Vec<Move>, non_quiescent_only: bool, mut pawn_occupancy: OccupancyBits, full_occupancy: OccupancyBits) {
        while pawn_occupancy != 0 {
            let (source_square_mask, source_square_shift) = mask_and_shift_from_lowest_one_bit(pawn_occupancy);
            pawn_occupancy &= !source_square_mask;

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
                    self.make_move(
                        result,
                        non_quiescent_only,
                        source_square_shift,
                        single_move_target_shift,
                        PAWN,
                        CASTLE_MOVE_FALSE_MASK,
                        EN_PASSANT_ATTACK_FALSE_MASK,
                        NO_PIECE,
                        NO_SQUARE,
                    );

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
                        self.make_move(
                            result,
                            non_quiescent_only,
                            source_square_shift,
                            double_move_target_mask.trailing_zeros(),
                            PAWN,
                            CASTLE_MOVE_FALSE_MASK,
                            EN_PASSANT_ATTACK_FALSE_MASK,
                            NO_PIECE,
                            single_move_target_shift,
                        );
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
                self.make_castle_move(result, E1, C1);
            }

            if self.white.king_side_castle
                && (full_occupancy & WHITE_KING_SIDE_CASTLE_EMPTY_OCCUPANCY) == 0
                && !self._is_occupancy_in_check(WHITE, &self.black, full_occupancy, WHITE_KING_SIDE_CASTLE_CHECK_OCCUPANCY) {
                self.make_castle_move(result, E1, G1);
            }
        } else {
            if self.black.queen_side_castle
                && ((full_occupancy & BLACK_QUEEN_SIDE_CASTLE_EMPTY_OCCUPANCY) == 0)
                && !self._is_occupancy_in_check(BLACK, &self.white, full_occupancy, BLACK_QUEEN_SIDE_CASTLE_CHECK_OCCUPANCY) {
                self.make_castle_move(result, E8, C8);
            }

            if self.black.king_side_castle
                && (full_occupancy & BLACK_KING_SIDE_CASTLE_EMPTY_OCCUPANCY) == 0
                && !self._is_occupancy_in_check(BLACK, &self.white, full_occupancy, BLACK_KING_SIDE_CASTLE_CHECK_OCCUPANCY) {
                self.make_castle_move(result, E8, G8);
            }
        }
    }

    #[inline(always)]
    fn make_castle_move(&self, result: &mut Vec<Move>, king_source_square_shift: SquareShiftBits, king_target_square_shift: SquareShiftBits) {
        self.make_move(
            result,
            false,
            king_source_square_shift,
            king_target_square_shift,
            KING,
            CASTLE_MOVE_TRUE_MASK,
            EN_PASSANT_ATTACK_FALSE_MASK,
            NO_PIECE,
            NO_SQUARE,
        );
    }

    fn generate_attacks(
        &self,
        result: &mut Vec<Move>,
        non_quiescent_only: bool,
        source_square_shift: SquareShiftBits,
        mut attack_occupancy: OccupancyBits,
        piece: PieceBits,
    ) {
        while attack_occupancy != 0 {
            let (target_square_mask, target_square_shift) = mask_and_shift_from_lowest_one_bit(attack_occupancy);
            attack_occupancy &= !target_square_mask;

            self.make_move(
                result,
                non_quiescent_only,
                source_square_shift,
                target_square_shift,
                piece,
                CASTLE_MOVE_FALSE_MASK,
                EN_PASSANT_ATTACK_FALSE_MASK,
                NO_PIECE,
                NO_SQUARE,
            );
        }
    }

    fn make_move(
        &self,
        result: &mut Vec<Move>,
        non_quiescent_only: bool,
        source_square_shift: SquareShiftBits,
        target_square_shift: SquareShiftBits,
        piece_active: PieceBits,
        is_castle_move_mask: u64,
        is_en_passant_attack_mask: u64,
        promote_to: PieceBits,
        en_passant_opportunity_square_shift: SquareShiftBits,
    ) {
        let active;
        let passive;
        let d_castle;
        let attack_square_shift;

        let en_passant_offset = if is_en_passant_attack_mask == 0 {
            0
        } else {
            8
        };

        if self.is_white_turn() {
            active = &self.white;
            passive = &self.black;
            d_castle = 0;
            attack_square_shift = target_square_shift + en_passant_offset;
        } else {
            active = &self.black;
            passive = &self.white;
            d_castle = 56;
            attack_square_shift = target_square_shift - en_passant_offset;
        }

        let piece_attacked = passive.get_piece_const_by_square_shift(attack_square_shift);

        if piece_attacked == NO_PIECE && promote_to == NO_PIECE && non_quiescent_only {
            return;
        }

        let mut mv = Move {
            bits: 0,
            mvvlva: 0,
        };

        mv.set_en_passant_attack(is_en_passant_attack_mask);

        mv.set_next_en_passant_square(en_passant_opportunity_square_shift);
        mv.set_piece_moved(piece_active);
        mv.set_piece_attacked(piece_attacked);
        mv.set_source_square(source_square_shift);
        mv.set_target_square(target_square_shift);
        mv.set_castle_move(is_castle_move_mask);
        mv.set_previous_halfmove(self.halfmove_clock);
        mv.set_previous_en_passant_square(self.en_passant_square_shift);
        mv.set_promotion_piece(promote_to);
        mv.set_side_to_move(self.turn);

        if piece_active == PAWN || piece_attacked != NO_PIECE {
            mv.set_halfmove_reset();
        }

        if passive.queen_side_castle && target_square_shift == (A8 + d_castle) {
            mv.set_opponent_lost_queen_side_castle();
        } else if passive.king_side_castle && target_square_shift == (H8 + d_castle) {
            mv.set_opponent_lost_king_side_castle();
        }

        if active.queen_side_castle && (source_square_shift == (A1 - d_castle) || source_square_shift == (E1 - d_castle)) {
            mv.set_self_lost_queen_side_castle();
        }

        if active.king_side_castle && (source_square_shift == (H1 - d_castle) || source_square_shift == (E1 - d_castle)) {
            mv.set_self_lost_king_side_castle();
        }

        mv.mvvlva = self.mvv_lva(piece_active, piece_attacked);
        result.push(mv);
    }

    const PIECE_VALUES: [i32; 7] = [
        0, 100, 320, 330, 500, 900, 901,
    ];

    fn mvv_lva(&self, piece_active: PieceBits, piece_attacked: PieceBits) -> i32 {
        if piece_attacked == NO_PIECE || piece_attacked == KING {
            return 0;
        }

        let active_value = Self::PIECE_VALUES[piece_active as usize];
        let target_value = Self::PIECE_VALUES[piece_attacked as usize];

        (target_value << 8) - active_value
    }
}

// Make/Unmake move
impl Bitboard {
    pub fn zobrist_xor(mv: Move) -> ZobristHash {
        let mut result: ZobristHash = 0;

        let self_color = mv.get_side_to_move();
        let opponent_color = opposite_color(self_color);

        result ^= Zobrist::BLACK_TO_MOVE_HASH;

        if mv.is_self_lost_king_side_castle() {
            result ^= Zobrist::castle_hash(KING, self_color);
        }

        if mv.is_self_lost_queen_side_castle() {
            result ^= Zobrist::castle_hash(QUEEN, self_color);
        }

        if mv.is_opponent_lost_king_side_castle() {
            result ^= Zobrist::castle_hash(KING, opponent_color);
        }

        if mv.is_opponent_lost_queen_side_castle() {
            result ^= Zobrist::castle_hash(QUEEN, opponent_color);
        }

        if mv.get_previous_en_passant_square() != NO_SQUARE {
            result ^= Zobrist::en_passant_square_hash(mv.get_previous_en_passant_square());
        }

        if mv.get_next_en_passant_square() != NO_SQUARE {
            result ^= Zobrist::en_passant_square_hash(mv.get_next_en_passant_square());
        }

        let is_white_turn = self_color == WHITE;

        let piece_moved = mv.get_piece_moved();
        let piece_promoted = mv.get_promotion_piece();
        let piece_attacked = mv.get_piece_attacked();
        let source_square_shift = mv.get_source_square();
        let target_square_shift = mv.get_target_square();

        if mv.is_castle_move() {
            let (rook_source_shift, king_source_shift, rook_target_shift, king_target_shift) = match target_square_shift {
                C1 => (A1, E1, D1, C1),
                G1 => (H1, E1, F1, G1),
                C8 => (A8, E8, D8, C8),
                G8 => (H8, E8, F8, G8),
                _ => panic!(),
            };

            result ^= Zobrist::piece_square_hash(ROOK, rook_source_shift, self_color);
            result ^= Zobrist::piece_square_hash(ROOK, rook_target_shift, self_color);
            result ^= Zobrist::piece_square_hash(KING, king_source_shift, self_color);
            result ^= Zobrist::piece_square_hash(KING, king_target_shift, self_color);
        } else if mv.is_en_passant_attack() {
            result ^= Zobrist::piece_square_hash(PAWN, source_square_shift, self_color);
            result ^= Zobrist::piece_square_hash(PAWN, target_square_shift, self_color);

            let pawn_target_shift = if is_white_turn {
                target_square_shift + 8
            } else {
                target_square_shift - 8
            };

            result ^= Zobrist::piece_square_hash(PAWN, pawn_target_shift, opponent_color);
        } else if mv.is_promotion() {
            result ^= Zobrist::piece_square_hash(PAWN, source_square_shift, self_color);
            result ^= Zobrist::piece_square_hash(piece_promoted, target_square_shift, self_color);
            result ^= Zobrist::piece_square_hash(piece_attacked, target_square_shift, opponent_color);
        } else {
            result ^= Zobrist::piece_square_hash(piece_moved, source_square_shift, self_color);
            result ^= Zobrist::piece_square_hash(piece_moved, target_square_shift, self_color);
            result ^= Zobrist::piece_square_hash(piece_attacked, target_square_shift, opponent_color);
        }

        result
    }

    pub fn make(&mut self, mv: Move) {
        let is_white_turn = self.is_white_turn();

        self.fullmove_clock += self.turn;

        if mv.is_halfmove_reset() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        self.en_passant_square_shift = mv.get_next_en_passant_square();

        self.turn = self.opposite_turn();

        // Swap active and passive because turn has already changed
        let (passive, active) = self.get_active_and_passive_mut();

        if mv.is_self_lost_king_side_castle() {
            active.king_side_castle = false;
        }

        if mv.is_self_lost_queen_side_castle() {
            active.queen_side_castle = false;
        }

        if mv.is_opponent_lost_king_side_castle() {
            passive.king_side_castle = false;
        }

        if mv.is_opponent_lost_queen_side_castle() {
            passive.queen_side_castle = false;
        }

        let source_square_shift = mv.get_source_square();
        let target_square_shift = mv.get_target_square();

        let source_square_mask: SquareMaskBits = 1_u64 << source_square_shift;
        let target_square_mask: SquareMaskBits = 1_u64 << target_square_shift;

        if mv.is_castle_move() {
            match target_square_shift {
                C1 => Self::make_castle(active, A1_MASK, source_square_mask, D1_MASK, target_square_mask),
                G1 => Self::make_castle(active, H1_MASK, source_square_mask, F1_MASK, target_square_mask),
                C8 => Self::make_castle(active, A8_MASK, source_square_mask, D8_MASK, target_square_mask),
                G8 => Self::make_castle(active, H8_MASK, source_square_mask, F8_MASK, target_square_mask),
                _ => panic!(),
            };
        } else if mv.is_en_passant_attack() {
            *active.pawns_ref() &= !source_square_mask;
            *active.pawns_ref() |= target_square_mask;

            let pawn_target_mask = if is_white_turn {
                target_square_mask << 8
            } else {
                target_square_mask >> 8
            };

            *passive.pawns_ref() &= !pawn_target_mask;
        } else if mv.is_promotion() {
            *active.pawns_ref() &= !source_square_mask;
            *active.occupancy_ref(mv.get_promotion_piece()) |= target_square_mask;
            *passive.occupancy_ref(mv.get_piece_attacked()) &= !target_square_mask;
            // passive.unset_all(target_square_mask);
        } else {
            *active.occupancy_ref(mv.get_piece_moved()) &= !source_square_mask;
            *active.occupancy_ref(mv.get_piece_moved()) |= target_square_mask;
            *passive.occupancy_ref(mv.get_piece_attacked()) &= !target_square_mask;
            // passive.unset_all(target_square_mask);
        }
    }


    pub fn unmake(&mut self, mv: Move) {
        let is_white_turn = self.is_white_turn();

        self.fullmove_clock -= 1 - self.turn;
        self.halfmove_clock = mv.get_previous_halfmove();
        self.en_passant_square_shift = mv.get_previous_en_passant_square();
        self.turn = self.opposite_turn();

        let (active, passive) = self.get_active_and_passive_mut();

        if mv.is_self_lost_king_side_castle() {
            active.king_side_castle = true;
        }

        if mv.is_self_lost_queen_side_castle() {
            active.queen_side_castle = true;
        }

        if mv.is_opponent_lost_king_side_castle() {
            passive.king_side_castle = true;
        }

        if mv.is_opponent_lost_queen_side_castle() {
            passive.queen_side_castle = true;
        }

        let source_square_shift = mv.get_source_square();
        let target_square_shift = mv.get_target_square();

        let source_square_mask: SquareMaskBits = 1_u64 << source_square_shift;
        let target_square_mask: SquareMaskBits = 1_u64 << target_square_shift;

        let piece_moved = mv.get_piece_moved();
        let piece_attacked = mv.get_piece_attacked();

        if mv.is_castle_move() {
            match target_square_shift {
                C1 => Self::unmake_castle(active, A1_MASK, source_square_mask, D1_MASK, target_square_mask),
                G1 => Self::unmake_castle(active, H1_MASK, source_square_mask, F1_MASK, target_square_mask),
                C8 => Self::unmake_castle(active, A8_MASK, source_square_mask, D8_MASK, target_square_mask),
                G8 => Self::unmake_castle(active, H8_MASK, source_square_mask, F8_MASK, target_square_mask),
                _ => panic!(),
            };
        } else if mv.is_en_passant_attack() {
            *active.pawns_ref() &= !target_square_mask;
            *active.pawns_ref() |= source_square_mask;

            let en_passant_attack_target_mask = if is_white_turn {
                target_square_mask >> 8
            } else {
                target_square_mask << 8
            };

            *passive.occupancy_ref(piece_attacked) |= en_passant_attack_target_mask;
        } else if mv.is_promotion() {
            *passive.occupancy_ref(piece_attacked) |= target_square_mask;
            *active.pawns_ref() |= source_square_mask;
            *active.occupancy_ref(mv.get_promotion_piece()) &= !target_square_mask;
        } else {
            *passive.occupancy_ref(piece_attacked) |= target_square_mask;
            *active.occupancy_ref(piece_moved) |= source_square_mask;
            *active.occupancy_ref(piece_moved) &= !target_square_mask;
        }
    }

    #[inline(always)]
    fn make_castle(
        active: &mut PlayerState,
        rook_source_mask: SquareMaskBits,
        king_source_mask: SquareMaskBits,
        rook_target_mask: SquareMaskBits,
        king_target_mask: SquareMaskBits,
    ) {
        *active.rooks_ref() &= !rook_source_mask;
        *active.kings_ref() &= !king_source_mask;

        *active.rooks_ref() |= rook_target_mask;
        *active.kings_ref() |= king_target_mask;
    }

    #[inline(always)]
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
        !self._is_in_check_by_bits(self.opposite_turn())
    }

    pub fn is_current_in_check(&self) -> bool {
        self._is_in_check_by_bits(self.turn)
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

        let full_occupancy = active.full_occupancy() | passive.full_occupancy();

        // Assume only one king
        self._is_square_in_check(color_bits, passive, active.kings().trailing_zeros(), full_occupancy)
    }

    fn _is_occupancy_in_check(&self, color_bits: ColorBits, passive: &PlayerState, full_occupancy: OccupancyBits, mut king_occupancy: OccupancyBits) -> bool {
        while king_occupancy != 0 {
            let (king_square_mask, king_square_shift) = mask_and_shift_from_lowest_one_bit(king_occupancy);
            king_occupancy &= !king_square_mask;

            if self._is_square_in_check(color_bits, passive, king_square_shift, full_occupancy) {
                return true;
            }
        }

        false
    }

    fn _is_square_in_check(&self, color_bits: ColorBits, passive: &PlayerState, king_square_shift: u32, full_occupancy: OccupancyBits) -> bool {
        let rook_attacks = ROOK_MAGICS.get_attacks(king_square_shift, full_occupancy);

        if (rook_attacks & (passive.rooks() | passive.queens())) != 0 {
            return true;
        }

        let bishop_attacks = BISHOP_MAGICS.get_attacks(king_square_shift, full_occupancy);

        if (bishop_attacks & (passive.bishops() | passive.queens())) != 0 {
            return true;
        }

        let knight_attacks = KNIGHT_NONMAGICS.get_attacks(king_square_shift);

        if (knight_attacks & passive.knights()) != 0 {
            return true;
        }

        let pawn_attacks = if color_bits == WHITE {
            WHITE_PAWN_NONMAGICS.get_attacks(king_square_shift)
        } else {
            BLACK_PAWN_NONMAGICS.get_attacks(king_square_shift)
        };

        if (pawn_attacks & passive.pawns()) != 0 {
            return true;
        }

        let king_attacks = KING_NONMAGICS.get_attacks(king_square_shift);

        (king_attacks & passive.kings()) != 0
    }
}

// Helpers
impl Bitboard {
    pub fn ply_clock(&self) -> u16 {
        (2 * (self.fullmove_clock - 1) + self.turn) as u16
    }

    #[inline(always)]
    fn is_white_turn(&self) -> bool {
        self.turn == WHITE
    }

    #[inline(always)]
    fn get_active_and_passive(&self) -> (&PlayerState, &PlayerState) {
        if self.is_white_turn() {
            (&self.white, &self.black)
        } else {
            (&self.black, &self.white)
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    fn opposite_turn(&self) -> ColorBits {
        opposite_color(self.turn)
    }

    pub fn find_uci(&mut self, uci: &str) -> Result<Move, MoveFromUciError> {
        let uci = uci.trim();
        let result = self.generate_pseudo_legal_moves().into_iter().find(|mv| mv.to_uci_string() == uci).ok_or_else(|| MoveDoesNotExist(uci.to_string()))?;

        self.make(result);
        if !self.is_valid() {
            return Err(MoveIsNotValid(result));
        }
        self.unmake(result);

        Ok(result)
    }

    pub fn make_uci(&mut self, uci: &str) -> Result<(), MoveFromUciError> {
        let mv = self.find_uci(uci)?;
        self.make(mv);

        Ok(())
    }

    pub fn make_all_uci(&mut self, moves: &[String]) -> Result<(), MoveFromUciError> {
        let mut potential_unmake = Vec::new();

        for uci in moves {
            match self.find_uci(uci) {
                Ok(mv) => {
                    self.make(mv);
                    potential_unmake.push(mv);
                }
                Err(error) => {
                    for mv in potential_unmake.iter().rev() {
                        self.unmake(*mv);
                    }
                    return Err(error);
                }
            };
        }

        Ok(())
    }

    pub fn uci_to_pgn(&mut self, uci: &str) -> Result<String, MoveFromUciError> {
        let uci = uci.trim();
        let moves = self.generate_pseudo_legal_moves();
        let result = *moves.iter().find(|mv| mv.to_uci_string() == uci).ok_or_else(|| MoveDoesNotExist(uci.to_string()))?;

        self.make(result);
        if !self.is_valid() {
            return Err(MoveIsNotValid(result));
        }

        let is_check = self.is_current_in_check();
        let is_mate = !self.is_any_move_legal(&self.generate_pseudo_legal_moves());
        self.unmake(result);


        let MoveStructs { from_square, to_square, from_piece, to_piece, promote_to } = MoveStructs::from(result);

        let legal_moves_with_same_to_square_and_same_piece: Vec<_> =
            moves
                .into_iter()
                .filter(|&mv| self.is_move_legal(mv))
                .filter(|mv| mv.get_target_square() == result.get_target_square())
                .filter(|mv| mv.get_piece_moved() == result.get_piece_moved())
                .collect();

        let any_share_source_rank =
            legal_moves_with_same_to_square_and_same_piece.iter()
                .any(|mv| {
                    let other_square = Square::by_index(mv.get_source_square() as usize).unwrap();
                    other_square.rank == from_square.rank && other_square.file != from_square.file
                });

        let any_share_source_file =
            legal_moves_with_same_to_square_and_same_piece.iter()
                .any(|mv| {
                    let other_square = Square::by_index(mv.get_source_square() as usize).unwrap();
                    other_square.file == from_square.file && other_square.rank != from_square.rank
                });


        let piece = if !matches!(from_piece, Piece::PAWN) {
            from_piece.as_white().fen.to_string()
        } else if to_piece.is_some() {
            from_square.file.fen.to_string()
        } else {
            "".to_string()
        };
        let is_pawn_move = from_piece == Piece::PAWN;

        let disambiguation_symbol = match (any_share_source_file, any_share_source_rank, is_pawn_move) {
            (true, _, true) => { from_square.file.fen.to_string() }
            (true, true, false) => { format!("{}{}", from_square.file.fen, from_square.rank.fen) }
            (true, false, false) => { from_square.rank.fen.to_string() }
            (false, true, false) => { from_square.file.fen.to_string() }
            (_, _, _) => { "".to_string() }
        };
        let capture = if to_piece.is_some() { "x" } else { "" };
        let target_square = to_square.fen();
        let promotion_piece = promote_to.map(|p| p.as_color(Color::WHITE));
        let promotion_piece = promotion_piece.map(|p| format!("={}", p.fen)).unwrap_or_else(|| "".to_string());
        let check_str = if is_mate { "#" } else if is_check { "+" } else { "" };

        if matches!(from_piece, Piece::KING) {
            let castle_move = match (from_square.file, to_square.file) {
                (File::FILE_E, File::FILE_G) => Some("O-O"),
                (File::FILE_E, File::FILE_C) => Some("O-O-O"),
                _ => None
            };

            if let Some(castle_move) = castle_move {
                return Ok(format!("{}{}", castle_move, check_str));
            }
        }

        Ok(format!("{}{}{}{}{}{}", piece, disambiguation_symbol, capture, target_square, promotion_piece, check_str))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_any_move_legal(&mut self, moves: &[Move]) -> bool {
        for &mv in moves {
            if self.is_move_legal(mv) {
                return true;
            }
        }

        false
    }

    #[inline(always)]
    #[allow(clippy::wrong_self_convention)]
    pub fn is_move_legal(&mut self, mv: Move) -> bool {
        self.make(mv);
        let result = self.is_valid();
        self.unmake(mv);

        result
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_any_move_non_quiescent(moves: &[Move]) -> bool {
        moves.iter().any(|mv| mv.is_attack() || mv.is_promotion())
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

        let mut buffer = Vec::new();
        self.generate_pseudo_legal_moves_with_buffer(&mut buffer);

        let mut next_buffer = Vec::new();
        for mv in buffer {
            self.make(mv);

            if self.is_valid() {
                result.push((mv, self._perft(&mut next_buffer, depth - 1)));
                next_buffer.clear();
            }

            self.unmake(mv);
        }

        result
    }

    fn _perft(&mut self, buffer: &mut Vec<Move>, depth: usize) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut count = 0;
        let mut next_buffer = Vec::new();
        self.generate_pseudo_legal_moves_with_buffer(buffer);
        for mv in buffer {
            self.make(*mv);

            if self.is_valid() {
                count += self._perft(&mut next_buffer, depth - 1);
                next_buffer.clear();
            }

            self.unmake(*mv);
        }

        count
    }

    pub fn calculate_zobrist_hash(&self) -> ZobristHash {
        Self::_zobrist_hash(&self.white, &self.black, self.turn, self.en_passant_square_shift)
    }

    fn _zobrist_hash(white: &PlayerState, black: &PlayerState, turn: ColorBits, en_passant_square_shift: SquareShiftBits) -> ZobristHash {
        let mut hash =
            Self::zobrist_hash_for_occupancy(white.kings(), KING, WHITE)
                ^ Self::zobrist_hash_for_occupancy(white.queens(), QUEEN, WHITE)
                ^ Self::zobrist_hash_for_occupancy(white.rooks(), ROOK, WHITE)
                ^ Self::zobrist_hash_for_occupancy(white.bishops(), BISHOP, WHITE)
                ^ Self::zobrist_hash_for_occupancy(white.knights(), KNIGHT, WHITE)
                ^ Self::zobrist_hash_for_occupancy(white.pawns(), PAWN, WHITE)
                ^ Self::zobrist_hash_for_occupancy(black.kings(), KING, BLACK)
                ^ Self::zobrist_hash_for_occupancy(black.queens(), QUEEN, BLACK)
                ^ Self::zobrist_hash_for_occupancy(black.rooks(), ROOK, BLACK)
                ^ Self::zobrist_hash_for_occupancy(black.bishops(), BISHOP, BLACK)
                ^ Self::zobrist_hash_for_occupancy(black.knights(), KNIGHT, BLACK)
                ^ Self::zobrist_hash_for_occupancy(black.pawns(), PAWN, BLACK)
            ;

        if turn == BLACK {
            hash ^= Zobrist::BLACK_TO_MOVE_HASH;
        }

        if en_passant_square_shift != NO_SQUARE {
            hash ^= Zobrist::en_passant_square_hash(en_passant_square_shift);
        }

        if white.queen_side_castle {
            hash ^= Zobrist::WHITE_QUEEN_CASTLE_HASH;
        }

        if white.king_side_castle {
            hash ^= Zobrist::WHITE_KING_CASTLE_HASH;
        }

        if black.queen_side_castle {
            hash ^= Zobrist::BLACK_QUEEN_CASTLE_HASH;
        }

        if black.king_side_castle {
            hash ^= Zobrist::BLACK_KING_CASTLE_HASH;
        }

        hash
    }

    fn zobrist_hash_for_occupancy(mut occupancy: OccupancyBits, piece: PieceBits, color: ColorBits) -> ZobristHash {
        let mut result = 0;

        while occupancy != 0 {
            let (mask, shift) = mask_and_shift_from_lowest_one_bit(occupancy);
            occupancy &= !mask;

            result ^= Zobrist::piece_square_hash(piece, shift, color);
        }

        result
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Move({}) {{ piece_moved = {}, piece_attacked = {}, self_lost_king_side_castle = {}, self_lost_queen_side_castle = {}, opponent_lost_king_side_castle = {}, opponent_lost_queen_side_castle = {}, castle_move = {}, en_passant_attack = {}, source_square = {}, target_square = {}, halfmove_reset = {}, previous_halfmove = {}, previous_en_passant_square = {}, next_en_passant_square = {}, promotion_piece = {}}}",
            self.to_uci_string(),
            piece_to_string(self.get_piece_moved()),
            piece_to_string(self.get_piece_attacked()),
            self.is_self_lost_king_side_castle(),
            self.is_self_lost_queen_side_castle(),
            self.is_opponent_lost_king_side_castle(),
            self.is_opponent_lost_queen_side_castle(),
            self.is_castle_move(),
            self.is_en_passant_attack(),
            square_to_string(self.get_source_square()),
            square_to_string(self.get_target_square()),
            self.is_halfmove_reset(),
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
            &"═══",
            board,
            other
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::{SliceRandom, StdRng};
    use rand::SeedableRng;

    use marvk_chess_core::fen::Fen;

    use crate::board::Bitboard;

    #[test]
    fn test_zobrist_consistency() {
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..1000 {
            let mut board = Bitboard::default();
            let mut zobrist_hash = board.calculate_zobrist_hash();

            for i in 1..200 {
                let mut moves = board.generate_legal_moves();

                for mv in &moves {
                    board.make(*mv);
                    let xor = Bitboard::zobrist_xor(*mv);
                    zobrist_hash ^= xor;
                    assert_eq!(zobrist_hash, board.calculate_zobrist_hash());
                    board.unmake(*mv);
                    zobrist_hash ^= xor;

                    assert_eq!(zobrist_hash, board.calculate_zobrist_hash());
                }

                moves.shuffle(&mut rng);

                if let Some(mv) = moves.first() {
                    let fen = board.fen().fen;
                    board.make(*mv);
                    let xor = Bitboard::zobrist_xor(*mv);
                    zobrist_hash ^= xor;

                    assert_eq!(zobrist_hash, board.calculate_zobrist_hash(), "failed after move #{}: {:?} --- fen: {}", i, mv, fen);
                } else {
                    break;
                }
            }
        }
    }


    #[test]
    fn test_zobrist_consistency_make_unmake() {
        for _ in 0..1 {}
    }

    #[test]
    fn test_ply_clock() {
        let mut board = Bitboard::default();

        for i in 0..100 {
            assert_eq!(board.ply_clock(), i);

            let moves = board.generate_legal_moves();

            board.make(*moves.first().unwrap());
        }
    }

    #[test]
    #[ignore]
    fn print_some_pgns() {
        let fen = Fen::new("r4rk1/ppqnpp1p/6pb/4p3/5P2/2N4Q/PPP2P1P/2KR3R b - - 1 16").unwrap();
        let mut board = Bitboard::new(&fen);

        for mv in board.generate_legal_moves() {
            println!("{}", mv.to_pgn_string(&mut board).unwrap());
        }
    }

    #[test]
    fn test_pgn1() {
        let fen = Fen::new("3q4/2P5/8/8/4Q2Q/k7/8/K6Q w - - 0 1").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("e4e1"), Ok("Qee1".to_string()));
        assert_eq!(board.uci_to_pgn("h4e1"), Ok("Qh4e1".to_string()));
        assert_eq!(board.uci_to_pgn("h1e1"), Ok("Q1e1".to_string()));
        assert_eq!(board.uci_to_pgn("c7c8q"), Ok("c8=Q".to_string()));
        assert_eq!(board.uci_to_pgn("c7d8n"), Ok("cxd8=N".to_string()));
    }

    #[test]
    fn test_pgn2() {
        let fen = Fen::new("8/8/5q2/4P1P1/8/k7/8/K7 w - - 0 1").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("e5f6"), Ok("exf6".to_string()));
        assert_eq!(board.uci_to_pgn("g5f6"), Ok("gxf6".to_string()));
    }

    #[test]
    fn test_pgn3() {
        let fen = Fen::new("8/8/8/1PpP4/8/k7/8/K7 w - c6 0 2").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("d5c6"), Ok("dxc6".to_string()));
        assert_eq!(board.uci_to_pgn("b5c6"), Ok("bxc6".to_string()));
    }

    #[test]
    fn test_pgn4() {
        let fen = Fen::new("1Q6/8/8/8/8/k1K1B3/8/8 w - - 0 1").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("b8a8"), Ok("Qa8#".to_string()));
        assert_eq!(board.uci_to_pgn("e3c5"), Ok("Bc5+".to_string()));
    }

    #[test]
    fn test_pgn5() {
        let fen = Fen::new("1q6/8/8/8/8/K1k1b3/8/8 b - - 0 1").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("b8a8"), Ok("Qa8#".to_string()));
        assert_eq!(board.uci_to_pgn("e3c5"), Ok("Bc5+".to_string()));
    }

    #[test]
    fn test_pgn_castle_white() {
        let fen = Fen::new("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("e1g1"), Ok("O-O".to_string()));
        assert_eq!(board.uci_to_pgn("e1c1"), Ok("O-O-O".to_string()));
    }

    #[test]
    fn test_pgn_castle_white_to_mate() {
        let fen = Fen::new("8/8/8/8/8/8/7R/k3K2R w K - 0 1").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("e1g1"), Ok("O-O#".to_string()));
    }

    #[test]
    fn test_pgn_castle_black() {
        let fen = Fen::new("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1").unwrap();
        let mut board = Bitboard::new(&fen);

        assert_eq!(board.uci_to_pgn("e8g8"), Ok("O-O".to_string()));
        assert_eq!(board.uci_to_pgn("e8c8"), Ok("O-O-O".to_string()));
    }

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
    fn test_black_in_check() {
        let fen_in_check = Fen::new("Q7/8/8/k1K5/8/8/8/8 b - - 2 1").unwrap();
        let board = Bitboard::new(&fen_in_check);

        assert!(board.is_current_in_check())
    }

    #[test]
    fn test_white_in_check() {
        let fen_in_check = Fen::new("q7/8/8/K1k5/8/8/8/8 w - - 1 1").unwrap();
        let board = Bitboard::new(&fen_in_check);

        assert!(board.is_current_in_check())
    }

    #[test]
    fn test_black_not_in_check() {
        let fen_in_check = Fen::new("1Q6/8/8/k1K5/8/8/8/8 b - - 2 1").unwrap();
        let board = Bitboard::new(&fen_in_check);

        assert!(!board.is_current_in_check())
    }

    #[test]
    fn test_white_not_in_check() {
        let fen_in_check = Fen::new("1q6/8/8/K1k5/8/8/8/8 w - - 1 1").unwrap();
        let board = Bitboard::new(&fen_in_check);

        assert!(!board.is_current_in_check())
    }
}

