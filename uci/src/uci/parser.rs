use std::cell::RefCell;
use std::cmp::max;
use std::collections::{HashSet, VecDeque};
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

use marvk_chess_core::fen::{Fen, FenParseError};

use crate::uci::{Go, ParseUciMoveError, UciMove};
use crate::uci::parser::ParserError::{DuplicatedToken, InvalidFen, InvalidInt, InvalidUciMove, UnexpectedEndOfCommand, UnexpectedToken, UnknownCommand};
use crate::uci::UciCommand;
use crate::uci::UciCommand::{Go as GoCommand, IsReady, PonderHit, PositionFrom, Quit, Register, RegisterLater, Stop, Uci, UciNewGame};

pub struct CommandParser<'a> {
    queue: RefCell<VecDeque<&'a str>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Node {
    key: String,
    value: NodeValue,
}

impl Node {
    const fn new(key: String, value: NodeValue) -> Self {
        Self { key, value }
    }

    pub fn empty(key: &str) -> Self {
        Self::new(key.to_string(), NodeValue::None)
    }

    pub fn single(key: &str, node: Self) -> Self {
        Self::new(key.to_string(), NodeValue::Single(Box::new(node)))
    }

    pub fn multiple(key: &str, nodes: Vec<Self>) -> Self {
        Self::new(key.to_string(), NodeValue::Multiple(nodes))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum NodeValue {
    None,
    Single(Box<Node>),
    Multiple(Vec<Node>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ParserError {
    UnknownCommand(String),
    UnexpectedEndOfCommand,
    UnexpectedToken { actual: String, expected: String },
    InvalidFen(FenParseError),
    InvalidInt(ParseIntError),
    DuplicatedToken(String),
    InvalidUciMove(ParseUciMoveError),
}

impl<'a> CommandParser<'a> {
    pub fn new(command: &'a str) -> Self {
        let queue = command.trim().split(' ').filter(|&s| !s.is_empty()).collect();

        Self { queue: RefCell::new(queue) }
    }

    pub fn parse(self) -> Result<UciCommand, ParserError> {
        self.parse_root(self.next()?)
    }

    fn parse_root(&self, root: &str) -> Result<UciCommand, ParserError> {
        match root {
            "uci" => Ok(Uci),
            "isready" => Ok(IsReady),
            "ucinewgame" => Ok(UciNewGame),
            "stop" => Ok(Stop),
            "ponderhit" => Ok(PonderHit),
            "quit" => Ok(Quit),
            "go" => self.parse_go(),
            "position" => self.parse_position(),
            "register" => self.parse_register(),
            "setoption" => self.parse_setoption(),
            "debug" => self.parse_debug(),
            _ => Err(UnknownCommand(root.to_string())),
        }
    }

    fn consume(&self, token: &str) -> Result<(), ParserError> {
        match self.next()? {
            actual if token == actual => Ok(()),
            actual => Err(UnexpectedToken { actual: actual.to_string(), expected: token.to_string() }),
        }
    }

    fn next(&self) -> Result<&str, ParserError> {
        self.queue.borrow_mut().pop_front().ok_or(UnexpectedEndOfCommand)
    }

    fn peek(&self) -> Result<&str, ParserError> {
        self.queue.borrow().front().copied().ok_or(UnexpectedEndOfCommand)
    }

    fn until_token_or_end(&self, token: &str) -> Result<String, ParserError> {
        self.until_one_of_or_end(&[token])
    }

    fn until_end(&self) -> Result<String, ParserError> {
        self.until_one_of_or_end(&[])
    }

    fn until_one_of_or_end(&self, stop_tokens: &[&str]) -> Result<String, ParserError> {
        let mut result = self.next()?.to_string();

        while self.peek().map(|s| !stop_tokens.contains(&s)).unwrap_or(false) {
            result.push(' ');
            result.push_str(self.next()?);
        }

        Ok(result)
    }

    const GO_TOKENS: [&'static str; 12] = ["searchmoves", "ponder", "wtime", "btime", "winc", "binc", "movestogo", "depth", "nodes", "mate", "movetime", "infinite"];

    fn parse_go(&self) -> Result<UciCommand, ParserError> {
        let mut go = Go::EMPTY;

        let mut visited_tokens = HashSet::new();

        loop {
            match self.next() {
                Ok(token) if visited_tokens.contains(token) => return Err(DuplicatedToken(token.to_string())),
                Ok(token) => {
                    match token {
                        "searchmoves" => go.search_moves = self.parse_moves_until_one_of_or_end(&Self::GO_TOKENS)?,
                        "ponder" => go.ponder = true,
                        "wtime" => go.white_time = self.parse_duration().map(Some)?,
                        "btime" => go.black_time = self.parse_duration().map(Some)?,
                        "winc" => go.white_increment = self.parse_duration().map(Some)?,
                        "binc" => go.black_increment = self.parse_duration().map(Some)?,
                        "movestogo" => go.moves_to_go = self.parse_u64().map(Some)?,
                        "depth" => go.depth = self.parse_u64().map(Some)?,
                        "nodes" => go.nodes = self.parse_u64().map(Some)?,
                        "mate" => go.mate = self.parse_u64().map(Some)?,
                        "movetime" => go.move_time = self.parse_duration().map(Some)?,
                        "infinite" => go.infinite = true,
                        _ => return Err(UnexpectedToken { actual: token.to_string(), expected: format!("one of {:?}", Self::GO_TOKENS) }),
                    }
                    visited_tokens.insert(token);
                }
                Err(UnexpectedEndOfCommand) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(GoCommand { go })
    }

    fn parse_duration(&self) -> Result<Duration, ParserError> { self.next()?.parse().map_err(InvalidInt).map(|d: i64| max(d, 0) as u64).map(Duration::from_millis) }
    fn parse_u64(&self) -> Result<u64, ParserError> { self.next()?.parse().map_err(InvalidInt) }

    fn parse_position(&self) -> Result<UciCommand, ParserError> {
        let fen = match self.next()? {
            "fen" => Fen::from_str(&self.until_token_or_end("moves")?).map_err(InvalidFen),
            "startpos" => Ok(Fen::default()),
            token => Err(UnexpectedToken { actual: token.to_string(), expected: format!("one of {:?}", &["fen", "startpos"]) })
        }?;

        let moves = match self.consume("moves") {
            Ok(_) => self.parse_moves(),
            Err(UnexpectedEndOfCommand) => Ok(Vec::new()),
            Err(error) => Err(error),
        }?;

        Ok(PositionFrom { fen, moves })
    }

    fn parse_moves(&self) -> Result<Vec<UciMove>, ParserError> {
        self.parse_moves_until_one_of_or_end(&[])
    }

    fn parse_moves_until_one_of_or_end(&self, stop_tokens: &[&str]) -> Result<Vec<UciMove>, ParserError> {
        let mut result = Vec::new();

        loop {
            match self.peek() {
                Ok(token) if stop_tokens.contains(&token) => break,
                Ok(_) => result.push(UciMove::from_str(self.next()?).map_err(InvalidUciMove)?),
                Err(UnexpectedEndOfCommand) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(result)
    }

    fn parse_register(&self) -> Result<UciCommand, ParserError> {
        if self.peek()? == "later" {
            Ok(RegisterLater)
        } else {
            self.consume("name")?;
            let name = self.until_token_or_end("code")?;
            self.consume("code")?;
            let code = self.until_end()?;
            Ok(Register { name, code })
        }
    }

    fn parse_setoption(&self) -> Result<UciCommand, ParserError> {
        self.consume("name")?;
        let name = self.until_token_or_end("value")?;
        let value_exists = self.consume("value");
        let value = self.until_end();

        match (value_exists, value) {
            (Ok(()), Ok(value)) => Ok(UciCommand::SetOptionValue { name, value }),
            (Err(UnexpectedEndOfCommand), _) => Ok(UciCommand::SetOption { name }),
            (Ok(()), Err(error)) | (Err(error), _) => Err(error),
        }
    }

    fn parse_debug(&self) -> Result<UciCommand, ParserError> {
        match self.next()? {
            "on" => Ok(true),
            "off" => Ok(false),
            token => Err(UnexpectedToken { actual: token.to_string(), expected: format!("one of {:?}", &["on", "off"]) }),
        }.map(|value| UciCommand::SetDebug { debug: value })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::time::Duration;

    use marvk_chess_core::constants::Piece;
    use marvk_chess_core::constants::Square;
    use marvk_chess_core::fen::Fen;
    use marvk_chess_core::fen::FenParseError::ConcurrentNumbers;

    use crate::uci::{ParseUciMoveError, UciCommand, UciMove};
    use crate::uci::Go;
    use crate::uci::parser::CommandParser;
    use crate::uci::parser::ParserError::{InvalidFen, InvalidUciMove, UnexpectedEndOfCommand, UnexpectedToken, UnknownCommand};
    use crate::uci::ParseUciMoveError::InvalidFormat;
    use crate::uci::UciCommand::{Go as GoCommand, IsReady, PonderHit, PositionFrom, Quit, Register, RegisterLater, SetDebug, SetOption, SetOptionValue, Stop, Uci, UciNewGame};

    #[test]
    fn general() {
        assert_eq!(CommandParser::new("").parse(), Err(UnexpectedEndOfCommand));
        assert_eq!(CommandParser::new("   ").parse(), Err(UnexpectedEndOfCommand));
        assert_eq!(CommandParser::new("something").parse(), Err(UnknownCommand("something".to_string())));
        assert_eq!(CommandParser::new("something   ").parse(), Err(UnknownCommand("something".to_string())));
        assert_eq!(CommandParser::new("").parse(), Err(UnexpectedEndOfCommand));
    }

    #[test]
    fn debug() {
        assert_eq!(CommandParser::new("debug on").parse(), Ok(SetDebug { debug: true }));
        assert_eq!(CommandParser::new("debug off").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new("debug off something").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new("debug off ").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new(" debug off ").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new("debug something").parse(), Err(UnexpectedToken { actual: "something".to_string(), expected: r#"one of ["on", "off"]"#.to_string() }));
        assert_eq!(CommandParser::new("debug ").parse(), Err(UnexpectedEndOfCommand));
        assert_eq!(CommandParser::new("debug").parse(), Err(UnexpectedEndOfCommand));
    }

    #[test]
    fn setoption() {
        assert_eq!(CommandParser::new("setoption name foo").parse(), Ok(SetOption { name: "foo".to_string() }));
        assert_eq!(CommandParser::new("setoption name foo ").parse(), Ok(SetOption { name: "foo".to_string() }));
        assert_eq!(CommandParser::new(" setoption name foo").parse(), Ok(SetOption { name: "foo".to_string() }));
        assert_eq!(CommandParser::new("setoption something foo").parse(), Err(UnexpectedToken { actual: "something".to_string(), expected: "name".to_string() }));
        assert_eq!(CommandParser::new("setoption name foo something").parse(), Ok(SetOption { name: "foo something".to_string() }));
        assert_eq!(CommandParser::new("setoption name foo value 1 2 3 ").parse(), Ok(SetOptionValue { name: "foo".to_string(), value: "1 2 3".to_string() }));
        assert_eq!(CommandParser::new("setoption name foo value  ").parse(), Err(UnexpectedEndOfCommand));
        assert_eq!(CommandParser::new("setoption   ").parse(), Err(UnexpectedEndOfCommand));
    }

    #[test]
    fn register() {
        assert_eq!(CommandParser::new("register").parse(), Err(UnexpectedEndOfCommand));
        assert_eq!(CommandParser::new("register later").parse(), Ok(RegisterLater));
        assert_eq!(CommandParser::new("   register later   something").parse(), Ok(RegisterLater));
        assert_eq!(CommandParser::new("register name Stefan MK code 4359874324").parse(), Ok(Register { name: "Stefan MK".to_string(), code: "4359874324".to_string() }));
        assert_eq!(CommandParser::new("  register name Stefan MK code 43598 74324 something  ").parse(), Ok(Register { name: "Stefan MK".to_string(), code: "43598 74324 something".to_string() }));
    }

    #[test]
    fn position() {
        assert_eq!(CommandParser::new("position fen").parse(), Err(UnexpectedEndOfCommand));
        assert_eq!(CommandParser::new("position fen rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2").parse(), Ok(PositionFrom { fen: Fen::from_str("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2").unwrap(), moves: Vec::new() }));
        assert_eq!(CommandParser::new("position fen rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2 moves").parse(), Ok(PositionFrom { fen: Fen::from_str("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2").unwrap(), moves: Vec::new() }));
        assert_eq!(CommandParser::new("position fen rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2 moves h4h6q a1a2").parse(), Ok(PositionFrom { fen: Fen::from_str("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2").unwrap(), moves: vec![UciMove::new_with_promotion(Square::H4, Square::H6, Piece::QUEEN), UciMove::new(Square::A1, Square::A2)] }));
        assert_eq!(CommandParser::new("position fen rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2 moves h4h6q a1a9").parse(), Err(InvalidUciMove(InvalidFormat("a1a9".to_string()))));
        assert_eq!(CommandParser::new("position fen rnbqkbnr/pp1ppppp/8/44/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2 moves h4h6q a1a9").parse(), Err(InvalidFen(ConcurrentNumbers { rank: "44".to_string() })));
        assert_eq!(CommandParser::new("position startpos").parse(), Ok(PositionFrom { fen: Fen::default(), moves: Vec::new() }));
        assert_eq!(CommandParser::new("position startpos moves").parse(), Ok(PositionFrom { fen: Fen::default(), moves: Vec::new() }));
        assert_eq!(CommandParser::new("position startpos moves h4h6q a1a2").parse(), Ok(PositionFrom { fen: Fen::default(), moves: vec![UciMove::new_with_promotion(Square::H4, Square::H6, Piece::QUEEN), UciMove::new(Square::A1, Square::A2)] }));
        assert_eq!(CommandParser::new("position startpos something").parse(), Err(UnexpectedToken { expected: "moves".to_string(), actual: "something".to_string() }));
    }

    #[test]
    fn go() {
        assert_eq!(CommandParser::new("go").parse(), Ok(GoCommand { go: Go::EMPTY }));
        assert_eq!(CommandParser::new("go searchmoves h4h6q a1a2 ponder wtime 60001 btime 60000 winc 1001 binc 1000 movestogo 10 depth 11 nodes 20000 mate 10 movetime 999 infinite").parse(),
                   Ok(GoCommand {
                       go: Go::new(
                           vec![UciMove::new_with_promotion(Square::H4, Square::H6, Piece::QUEEN), UciMove::new(Square::A1, Square::A2)],
                           true,
                           Some(Duration::from_millis(60001)),
                           Some(Duration::from_millis(60000)),
                           Some(Duration::from_millis(1001)),
                           Some(Duration::from_millis(1000)),
                           Some(10),
                           Some(11),
                           Some(20000),
                           Some(10),
                           Some(Duration::from_millis(999)),
                           true,
                       )
                   })
        );
        assert_eq!(CommandParser::new("go searchmoves h4h6q a1a2 wtime 60001 btime 60000 winc 1001 binc 1000 movestogo 10 depth 11 nodes 20000 mate 10 movetime 999").parse(),
                   Ok(GoCommand {
                       go: Go::new(
                           vec![UciMove::new_with_promotion(Square::H4, Square::H6, Piece::QUEEN), UciMove::new(Square::A1, Square::A2)],
                           false,
                           Some(Duration::from_millis(60001)),
                           Some(Duration::from_millis(60000)),
                           Some(Duration::from_millis(1001)),
                           Some(Duration::from_millis(1000)),
                           Some(10),
                           Some(11),
                           Some(20000),
                           Some(10),
                           Some(Duration::from_millis(999)),
                           false,
                       )
                   })
        );
        assert_eq!(CommandParser::new(" go    searchmoves h4h6q a1a2 wtime 60001 winc 1001  btime 60000 binc 1000 movestogo 10 depth 11 nodes 20000 mate 10 movetime 999").parse(),
                   Ok(GoCommand {
                       go: Go::new(
                           vec![UciMove::new_with_promotion(Square::H4, Square::H6, Piece::QUEEN), UciMove::new(Square::A1, Square::A2)],
                           false,
                           Some(Duration::from_millis(60001)),
                           Some(Duration::from_millis(60000)),
                           Some(Duration::from_millis(1001)),
                           Some(Duration::from_millis(1000)),
                           Some(10),
                           Some(11),
                           Some(20000),
                           Some(10),
                           Some(Duration::from_millis(999)),
                           false,
                       )
                   })
        );
        assert_eq!(CommandParser::new(" go    searchmoves h4h6q a1a2 wtime 60001 winc 1001  btime 60000 binc 1000 movestogo 10 depth 11 nodes 20000 mate 10 movetime 999  something").parse(), Err(UnexpectedToken { actual: "something".to_string(), expected: format!("one of {:?}", CommandParser::GO_TOKENS) }));
        assert_eq!(CommandParser::new("go searchmoves h4h6x").parse(), Err(InvalidUciMove(ParseUciMoveError::InvalidFormat("h4h6x".to_string()))));
        assert_eq!(CommandParser::new("go btime -60000").parse(), Ok(GoCommand { go: Go { black_time: Some(Duration::from_millis(0)), ..Go::EMPTY } }));
    }

    #[test]
    fn uci() {
        run_test_for_simple_command("uci", Uci);
    }

    #[test]
    fn isready() {
        run_test_for_simple_command("isready", IsReady);
    }

    #[test]
    fn ucinewgame() {
        run_test_for_simple_command("ucinewgame", UciNewGame);
    }

    #[test]
    fn stop() {
        run_test_for_simple_command("stop", Stop);
    }

    #[test]
    fn ponderhit() {
        run_test_for_simple_command("ponderhit", PonderHit);
    }

    #[test]
    fn quit() {
        run_test_for_simple_command("quit", Quit);
    }

    fn run_test_for_simple_command(input: &str, expected: UciCommand) {
        let expected = Ok(expected);
        assert_eq!(CommandParser::new(input).parse(), expected);
        assert_eq!(CommandParser::new(&format!(" {}", input)).parse(), expected);
        assert_eq!(CommandParser::new(&format!("{} ", input)).parse(), expected);
        assert_eq!(CommandParser::new(&format!("{} something", input)).parse(), expected);
    }
}
