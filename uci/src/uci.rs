use std::fmt::{Display, Formatter};
use std::time::Duration;

use marvk_chess_core::constants::piece::Piece;
use marvk_chess_core::constants::square::*;
use marvk_chess_core::fen::Fen;

use crate::uci::ParseUciMoveError::InvalidFormat;

pub mod console;
pub mod parser;
pub mod command;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseUciMoveError {
    InvalidFormat(String)
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UciMove {
    pub source: Square,
    pub target: Square,
    pub promote_to: Option<Piece>,
}

impl UciMove {
    pub fn new(source: Square, target: Square) -> Self {
        Self { source, target, promote_to: None }
    }

    pub fn new_with_promotion(source: Square, target: Square, promote_to: Piece) -> Self {
        Self { source, target, promote_to: Some(promote_to) }
    }

    pub fn parse(raw: &str) -> Result<UciMove, ParseUciMoveError> {
        let mut chars = raw.chars();

        let produce_error = || InvalidFormat(raw.to_string());

        let mut next_char = || {
            chars.next().ok_or_else(produce_error)
        };

        let source = Square::by_chars(next_char()?, next_char()?).ok_or_else(produce_error)?;
        let target = Square::by_chars(next_char()?, next_char()?).ok_or_else(produce_error)?;

        let promote_to = match next_char() {
            Ok(c) => Some(Piece::by_char(c).ok_or_else(produce_error)?),
            Err(_) => None,
        };

        Ok(UciMove {
            source,
            target,
            promote_to,
        })
    }
}

impl Display for UciMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.source.fen(),
            self.target.fen(),
            self.promote_to.as_ref().map(|m| m.uci_name().to_string()).unwrap_or_else(|| "".to_string())
        )
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Go {
    pub search_moves: Vec<UciMove>,
    pub ponder: bool,
    pub white_time: Option<Duration>,
    pub black_time: Option<Duration>,
    pub white_increment: Option<Duration>,
    pub black_increment: Option<Duration>,
    pub moves_to_go: Option<u64>,
    pub depth: Option<u64>,
    pub nodes: Option<u64>,
    pub mate: Option<u64>,
    pub move_time: Option<Duration>,
    pub infinite: bool,
}

impl Go {
    pub const EMPTY: Go = Go::new(Vec::new(), false, None, None, None, None, None, None, None, None, None, false);

    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        search_moves: Vec<UciMove>,
        ponder: bool,
        white_time: Option<Duration>,
        black_time: Option<Duration>,
        white_increment: Option<Duration>,
        black_increment: Option<Duration>,
        moves_to_go: Option<u64>,
        depth: Option<u64>,
        nodes: Option<u64>,
        mate: Option<u64>,
        move_time: Option<Duration>,
        infinite: bool
    ) -> Self {
        Self { search_moves, ponder, white_time, black_time, white_increment, black_increment, moves_to_go, depth, nodes, mate, move_time, infinite }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Bound {
    LOWER,
    UPPER,
}

impl Display for Bound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Bound::LOWER => "lowerbound",
                Bound::UPPER => "upperbound",
            },
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Info {
    pub depth: Option<u32>,
    pub selective_depth: Option<u32>,
    pub time: Option<Duration>,
    pub nodes: Option<u64>,
    pub principal_variation: Option<Vec<UciMove>>,
    pub multi_pv: Option<u32>,
    pub score: Option<Score>,
    pub current_move: Option<UciMove>,
    pub current_move_number: Option<u32>,
    pub hash_full: Option<u32>,
    pub nps: Option<u64>,
    pub table_hits: Option<u32>,
    pub shredder_table_hits: Option<u32>,
    pub cpu_load: Option<u32>,
    pub string: Option<String>,
    pub refutation: Option<Vec<UciMove>>,
    pub current_line: Option<CurrentLine>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CurrentLine {
    cpu_number: u32,
    line: Vec<UciMove>,
}

impl CurrentLine {
    pub fn new(cpu_number: u32, line: Vec<UciMove>) -> Self {
        Self { cpu_number, line }
    }
}

impl Info {
    #[allow(clippy::too_many_arguments)]
    pub fn new(depth: u32, selective_depth: u32, time: Duration, nodes: u64, principal_variation: Vec<UciMove>, multi_pv: u32, score: Score, current_move: UciMove, current_move_number: u32, hash_full: u32, nps: u64, table_hits: u32, shredder_table_hits: u32, cpu_load: u32, string: String, refutation: Vec<UciMove>, current_line_cpu_number: u32, current_line: Vec<UciMove>) -> Self {
        Self {
            depth: Some(depth),
            selective_depth: Some(selective_depth),
            time: Some(time),
            nodes: Some(nodes),
            principal_variation: Some(principal_variation),
            multi_pv: Some(multi_pv),
            score: Some(score),
            current_move: Some(current_move),
            current_move_number: Some(current_move_number),
            hash_full: Some(hash_full),
            nps: Some(nps),
            table_hits: Some(table_hits),
            shredder_table_hits: Some(shredder_table_hits),
            cpu_load: Some(cpu_load),
            string: Some(string),
            refutation: Some(refutation),
            current_line: Some(CurrentLine::new(current_line_cpu_number, current_line)),
        }
    }

    pub const EMPTY: Self = Info {
        depth: None,
        selective_depth: None,
        time: None,
        nodes: None,
        principal_variation: None,
        multi_pv: None,
        score: None,
        current_move: None,
        current_move_number: None,
        hash_full: None,
        nps: None,
        table_hits: None,
        shredder_table_hits: None,
        cpu_load: None,
        string: None,
        refutation: None,
        current_line: None,
    };
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Score {
    Centipawn { score: i32 },
    CentipawnBounded { score: i32, bound: Bound },
    Mate { mate_in: i32 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProtectionMessage {
    CHECKING,
    OK,
    ERROR,
}

impl Display for ProtectionMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ProtectionMessage::CHECKING => "checking",
                ProtectionMessage::OK => "ok",
                ProtectionMessage::ERROR => "error",
            },
        )
    }
}

// pub trait UciRx {
//     fn uci(&self);
//     fn set_debug(&self, debug: bool);
//     fn is_ready(&self);
//     fn set_option(&self, name: &str, value: &str);
//     fn register_later(&self);
//     fn register(&self, name: &str, code: &str);
//     fn uci_new_game(&self);
//     fn position_from_default(&self, uci_moves: &[UciMove]);
//     fn position_from(&self, fen: Fen, uci_moves: &[UciMove]);
//     fn go(&self, go: &Go);
//     fn stop(&self);
//     fn ponder_hit(&self);
//     fn quit(&self);
// }

#[derive(Debug, Eq, PartialEq)]
pub enum UciCommand {
    Uci,
    SetDebug { debug: bool },
    IsReady,
    SetOption { name: String },
    SetOptionValue { name: String, value: String },
    RegisterLater,
    Register { name: String, code: String },
    UciNewGame,
    PositionFrom { fen: Fen, moves: Vec<UciMove> },
    Go { go: Go },
    Stop,
    PonderHit,
    Quit,
}

#[derive(Debug, Eq, PartialEq)]
pub enum UciTxCommand {
    IdName { name: String },
    IdAuthor { author: String },
    Ok,
    ReadyOk,
    BestMove { best_move: Option<UciMove>, ponder_move: Option<UciMove> },
    CopyProtection { copy_protection: ProtectionMessage },
    Registration { registration: ProtectionMessage },
    Info { info: Info },
    OptionCheck { name: String, default: bool },
    OptionSpin { name: String, default: i32, min: i32, max: i32 },
    OptionCombo { name: String, default: String, vars: Vec<String> },
    OptionButton { name: String },
    OptionString { name: String, default: String },
    Debug { message: String },
}

impl UciCommand {}

pub trait Engine {
    fn accept(&mut self, command: UciCommand);
}

pub trait UciTx {
    fn id_name(&self, name: &str);
    fn id_author(&self, author: &str);
    fn uci_ok(&self);
    fn ready_ok(&self);
    fn best_move(&self, uci_move: Option<UciMove>, ponder_uci_move: Option<UciMove>);
    fn copy_protection(&self, copy_protection: ProtectionMessage);
    fn registration(&self, registration: ProtectionMessage);
    fn info(&self, info: &Info);
    fn option_check(&self, name: &str, default: bool);
    fn option_spin(&self, name: &str, default: i32, min: i32, max: i32);
    fn option_combo(&self, name: &str, default: &str, vars: &[&str]);
    fn option_button(&self, name: &str);
    fn option_string(&self, name: &str, default: &str);
    fn debug(&self, message: &str);
}

#[cfg(test)]
mod tests {
    use marvk_chess_core::constants::piece::Piece;
    use marvk_chess_core::constants::square::Square;

    use crate::uci::{ParseUciMoveError, UciMove};

    #[test]
    fn test_parse_uci_move() {
        assert_eq!(UciMove::parse("a1a2"), Ok(UciMove::new(Square::A1, Square::A2)));
        assert_eq!(UciMove::parse("a8h8"), Ok(UciMove::new(Square::A8, Square::H8)));
        assert_eq!(UciMove::parse("h1a1"), Ok(UciMove::new(Square::H1, Square::A1)));
        assert_eq!(UciMove::parse("h1a1q"), Ok(UciMove::new_with_promotion(Square::H1, Square::A1, Piece::QUEEN)));
        assert_eq!(UciMove::parse("h1a1k"), Ok(UciMove::new_with_promotion(Square::H1, Square::A1, Piece::KING)));
        assert_eq!(UciMove::parse("h1a0k"), Err(ParseUciMoveError::InvalidFormat("h1a0k".to_string())));
        assert_eq!(UciMove::parse("h1a1v"), Err(ParseUciMoveError::InvalidFormat("h1a1v".to_string())));
        assert_eq!(UciMove::parse("x1a1"), Err(ParseUciMoveError::InvalidFormat("x1a1".to_string())));
    }
}
