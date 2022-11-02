use std::fmt::{Display, Formatter};
use std::time::Duration;

use marvk_chess_core::constants::piece::Piece;
use marvk_chess_core::constants::square::*;
use marvk_chess_core::fen::Fen;

pub mod console;
pub mod parser;

#[derive(Debug, Eq, PartialEq)]
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
}

impl Display for UciMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.source.fen(),
            self.target.fen(),
            self.promote_to.as_ref().map(|m| m.uci_name().to_string()).unwrap_or("".to_string())
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Go {
    search_moves: Option<Vec<UciMove>>,
    ponder: bool,
    white_time: Option<Duration>,
    black_time: Option<Duration>,
    white_increment: Option<Duration>,
    black_increment: Option<Duration>,
    moves_to_go: Option<u32>,
    depth: Option<u32>,
    nodes: Option<u64>,
    mate: Option<u32>,
    move_time: Option<Duration>,
    infinite: bool,
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

pub struct Info<'a> {
    depth: Option<u32>,
    selective_depth: Option<u32>,
    time: Option<Duration>,
    nodes: Option<u64>,
    principal_variation: Option<&'a [UciMove]>,
    multi_pv: Option<u32>,
    score: Option<Score>,
    current_move: Option<UciMove>,
    current_move_number: Option<u32>,
    hash_full: Option<u32>,
    nps: Option<u64>,
    table_hits: Option<u32>,
    shredder_table_hits: Option<u32>,
    cpu_load: Option<u32>,
    string: Option<&'a str>,
    refutation: Option<&'a [UciMove]>,
    current_line: Option<CurrentLine<'a>>,
}

pub struct CurrentLine<'a> {
    cpu_number: u32,
    line: &'a [UciMove],
}

impl<'a> CurrentLine<'a> {
    pub fn new(cpu_number: u32, line: &'a [UciMove]) -> Self {
        Self { cpu_number, line }
    }
}

impl<'a> Info<'a> {
    pub fn new(depth: u32, selective_depth: u32, time: Duration, nodes: u64, principal_variation: &'a [UciMove], multi_pv: u32, score: Score, current_move: UciMove, current_move_number: u32, hash_full: u32, nps: u64, table_hits: u32, shredder_table_hits: u32, cpu_load: u32, string: &'a str, refutation: &'a [UciMove], current_line_cpu_number: u32, current_line: &'a [UciMove]) -> Self {
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
    PositionFromDefault { moves: Vec<UciMove> },
    PositionFrom { fen: Fen, moves: Vec<UciMove> },
    Go { go: Go },
    Stop,
    PonderHit,
    Quit,
}

impl UciCommand {

}

pub trait Engine {
    fn accept(&self, command: UciCommand);
}

pub trait UciTx {
    fn id_name(&self, name: &str);
    fn id_author(&self, author: &str);
    fn uci_ok(&self);
    fn ready_ok(&self);
    fn best_move(&self, uci_move: &UciMove);
    fn best_move_with_ponder(&self, uci_move: &UciMove, ponder_uci_move: &UciMove);
    fn copy_protection(&self, copy_protection: ProtectionMessage);
    fn registration(&self, registration: ProtectionMessage);
    fn info(&self, info: &Info);
}


