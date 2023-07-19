use std::ops::Range;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::fen::ParseFenError::{ConcurrentNumbers, RankWithInvalidPieceCount};

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Fen {
    pub fen: String,
    piece_placement: Range<usize>,
    active_color: Range<usize>,
    castling_availability: Range<usize>,
    en_passant_target_square: Range<usize>,
    halfmove_clock: Option<Range<usize>>,
    fullmove_clock: Option<Range<usize>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseFenError {
    InvalidCapture(String),
    RankWithInvalidPieceCount { rank: String, count: u32 },
    IllegalNumberOfGroups(usize),
    ConcurrentNumbers { rank: String },
}

lazy_static! {
    pub static ref FEN_STARTPOS: Fen = Fen::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
}

impl Fen {
    pub fn is_valid(fen: &str) -> bool {
        Self::new(fen).is_ok()
    }

    pub fn new(fen: &str) -> Result<Fen, ParseFenError> {
        if fen == "startpos" {
            return Ok(FEN_STARTPOS.clone());
        }

        let fen = fen.to_string();
        let temp_fen = fen.clone();
        let captures = Self::parse(&temp_fen)?;

        let group_to_slice = |match_index| {
            captures.get(match_index).map(|m| {
                m.range()
            })
        };

        Self::validate_ranks(group_to_slice(1).map(|range| &fen[range.start..range.end]).unwrap())?;

        Ok(
            Self {
                fen,
                piece_placement: group_to_slice(1).unwrap(),
                active_color: group_to_slice(2).unwrap(),
                castling_availability: group_to_slice(3).unwrap(),
                en_passant_target_square: group_to_slice(4).unwrap(),
                halfmove_clock: group_to_slice(5),
                fullmove_clock: group_to_slice(6),
            }
        )
    }

    pub fn get_piece_placement(&self) -> &str {
        &self.fen[self.piece_placement.start..self.piece_placement.end]
    }
    pub fn get_active_color(&self) -> &str {
        &self.fen[self.active_color.start..self.active_color.end]
    }
    pub fn get_castling_availability(&self) -> &str {
        &self.fen[self.castling_availability.start..self.castling_availability.end]
    }
    pub fn get_en_passant_target_square(&self) -> &str {
        &self.fen[self.en_passant_target_square.start..self.en_passant_target_square.end]
    }
    pub fn get_halfmove_clock(&self) -> &str {
        match &self.halfmove_clock {
            Some(range) => &self.fen[range.start..range.end],
            None => "0",
        }
    }
    pub fn get_fullmove_clock(&self) -> &str {
        match &self.fullmove_clock {
            Some(range) => &self.fen[range.start..range.end],
            None => "1",
        }
    }

    fn parse(fen: &str) -> Result<Captures, ParseFenError> {
        lazy_static! {
            static ref FEN_REGEX: Regex = Regex::new(r"^([PNBRQKpnbrqk1-8]{1,8}(?:/[PNBRQKpnbrqk1-8]{1,8}){7}) ([bw]) (KQ?k?q?|Qk?q?|kq?|q|-) ([a-h][1-8]|-)(?: (\d+) (\d+))?$").unwrap();
        }

        match FEN_REGEX.captures(fen) {
            Some(captures) if (captures.len() == 7 || captures.len() == 5) => Ok(captures),
            Some(captures) => Err(ParseFenError::IllegalNumberOfGroups(captures.len())),
            None => Err(ParseFenError::InvalidCapture(fen.to_string())),
        }
    }

    fn validate_ranks(ranks: &str) -> Result<(), ParseFenError> {
        ranks
            .split("/")
            .map(Self::validate_rank)
            .find(Result::is_err)
            .unwrap_or(Ok(()))
    }

    fn validate_rank(rank: &str) -> Result<(), ParseFenError> {
        let count: u32 = rank.chars().map(|c| c.to_digit(10).unwrap_or(1)).sum();

        if count != 8 {
            return Err(RankWithInvalidPieceCount { rank: rank.to_string(), count });
        }

        let chars = rank.chars().collect::<Vec<char>>();

        for i in 0..rank.len() - 1 {
            if chars[i].is_ascii_digit() && chars[i + 1].is_ascii_digit() {
                return Err(ConcurrentNumbers { rank: rank.to_string() });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fen::{Fen, ParseFenError};

    #[derive(Debug, Eq, PartialEq)]
    struct ExtractedFen {
        fen: String,
        piece_placement: String,
        active_color: String,
        castling_availability: String,
        en_passant_target_square: String,
        halfmove_clock: String,
        fullmove_clock: String,
    }

    impl ExtractedFen {
        pub fn new(fen: &str, piece_placement: &str, active_color: &str, castling_availability: &str, en_passant_target_square: &str, halfmove_clock: &str, fullmove_clock: &str) -> Self {
            Self {
                fen: fen.to_string(),
                piece_placement: piece_placement.to_string(),
                active_color: active_color.to_string(),
                castling_availability: castling_availability.to_string(),
                en_passant_target_square: en_passant_target_square.to_string(),
                halfmove_clock: halfmove_clock.to_string(),
                fullmove_clock: fullmove_clock.to_string(),
            }
        }
    }

    #[test]
    fn fen_ok_1() {
        test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            Ok(ExtractedFen::new(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
                "w",
                "KQkq",
                "-",
                "0",
                "1",
            )),
        )
    }

    #[test]
    fn fen_ok_2() {
        test(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            Ok(ExtractedFen::new(
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR",
                "b",
                "KQkq",
                "e3",
                "0",
                "1",
            )),
        )
    }

    #[test]
    fn fen_ok_3() {
        test(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
            Ok(ExtractedFen::new(
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR",
                "w",
                "KQkq",
                "c6",
                "0",
                "2",
            )),
        )
    }

    #[test]
    fn fen_ok_4() {
        test(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
            Ok(ExtractedFen::new(
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R",
                "b",
                "-",
                "-",
                "1",
                "2",
            )),
        );
    }

    #[test]
    fn fen_ok_5() {
        test(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
            Ok(ExtractedFen::new(
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
                "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R",
                "b",
                "-",
                "-",
                "1",
                "2",
            )),
        )
    }

    #[test]
    fn fen_err_1() {
        test(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b -  1 2",
            Err(ParseFenError::InvalidCapture("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b -  1 2".to_string())),
        )
    }

    #[test]
    fn fen_err_2() {
        test(
            "rnbqbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
            Err(ParseFenError::RankWithInvalidPieceCount { rank: "rnbqbnr".to_string(), count: 7 }),
        )
    }

    #[test]
    fn fen_err_3() {
        test(
            "rnbqkbnr/pp1ppppp/8/2p4/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
            Err(ParseFenError::RankWithInvalidPieceCount { rank: "2p4".to_string(), count: 7 }),
        )
    }

    #[test]
    fn fen_err_4() {
        test(
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - 1 2",
            Err(ParseFenError::InvalidCapture("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - 1 2".to_string())),
        )
    }

    #[test]
    fn fen_err_5() {
        test(
            "rnbqkbnr/pp1ppppp/44/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2",
            Err(ParseFenError::ConcurrentNumbers { rank: "44".to_string() }),
        )
    }

    fn test(fen_string: &str, expected: Result<ExtractedFen, ParseFenError>) {
        assert_eq!(Fen::new(fen_string).map(|fen| {
            println!("{:?}", fen);
            ExtractedFen::new(
                fen.fen.as_str(),
                fen.get_piece_placement(),
                fen.get_active_color(),
                fen.get_castling_availability(),
                fen.get_en_passant_target_square(),
                fen.get_halfmove_clock(),
                fen.get_fullmove_clock(),
            )
        }), expected);
        assert_eq!(Fen::is_valid(fen_string), expected.is_ok());
    }
}
