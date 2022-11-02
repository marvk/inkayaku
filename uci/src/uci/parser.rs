use std::cell::{RefCell, RefMut};
use std::collections::VecDeque;

use crate::uci::{Go, UciCommand};
use crate::uci::parser::ParserError::*;
use crate::uci::UciCommand::{IsReady, PonderHit, Quit, Stop, Uci, UciNewGame};

pub struct CommandParser<'a> {
    queue: RefCell<VecDeque<&'a str>>,
    current_token: RefCell<Option<&'a str>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Node {
    key: String,
    value: NodeValue,
}

impl Node {
    fn new(key: String, value: NodeValue) -> Self {
        Self { key, value }
    }

    pub fn empty(key: &str) -> Self {
        Self::new(key.to_string(), NodeValue::None)
    }

    pub fn single(key: &str, node: Node) -> Self {
        Self::new(key.to_string(), NodeValue::Single(Box::new(node)))
    }

    pub fn multiple(key: &str, nodes: Vec<Node>) -> Self {
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
    EmptyCommand,
    InvalidToken(String),
    AlreadyConsumed,
    EmptyToken,
    EmptyRemaining,
    UnexpectedEndOfCommand,
    UnknownCommand(String),
}

impl<'a> CommandParser<'a> {
    pub fn new(command: &'a str) -> Self {
        let mut queue: VecDeque<&str> = command.trim().split(' ').filter(|&s| !s.is_empty()).collect();
        let next = queue.pop_front();

        Self { queue: RefCell::new(queue), current_token: RefCell::new(next) }
    }

    pub fn parse(self) -> Result<UciCommand, ParserError> {
        let option = *self.current_token.borrow();

        if let Some(command) = option {
            self.parse_root(command)
        } else {
            Err(EmptyCommand)
        }
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

    }

    fn next(&self) -> Result<&str, ParserError> {
        let current = self.queue.borrow_mut().pop_front();
        self.current_token.replace(current);
        match current {
            Some(token) => Ok(token),
            None => Err(EmptyToken),
        }
    }

    fn peek(&self) -> Option<&str> {
        self.queue.borrow().front().map(|&s| s)
    }

    fn remaining(&self) -> Result<String, ParserError> {
        match self.queue.borrow().iter().cloned().collect::<Vec<&str>>().join(" ") {
            result if !result.is_empty() => Ok(result),
            _ => Err(EmptyRemaining)
        }
    }

    fn single(&self, key: &str) -> Result<&str, ParserError> {
        match self.next()? {
            token if token == key => Ok(self.next()?),
            token => Err(InvalidToken(token.to_string())),
        }
    }

    fn single_with_remaining(&self, key: &str) -> Result<String, ParserError> {
        match self.next()? {
            token if token == key => Ok(self.remaining()?),
            token => Err(InvalidToken(token.to_string())),
        }
    }

    fn parse_go(&self) -> Result<UciCommand, ParserError> {
        todo!()
    }

    fn parse_position(&self) -> Result<UciCommand, ParserError> {
        todo!()
    }

    fn parse_register(&self) -> Result<UciCommand, ParserError> {
        todo!()
    }

    fn parse_setoption(&self) -> Result<UciCommand, ParserError> {
        let name = self.single("name")?.to_string();

        match self.soft_optional(self.single_with_remaining("value"))? {
            Some(value) => Ok(UciCommand::SetOptionValue { name, value }),
            None => Ok(UciCommand::SetOption { name }),
        }
    }

    fn soft_optional<T>(&self, result: Result<T, ParserError>) -> Result<Option<T>, ParserError> {
        match result {
            Ok(node) => Ok(Some(node)),
            Err(UnexpectedEndOfCommand) | Err(EmptyToken) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn soft_push(&self, vec: &mut Vec<Node>, result: Result<Node, ParserError>) -> Result<(), ParserError> {
        match result {
            Ok(node) => vec.push(node),
            Err(UnexpectedEndOfCommand) => {}
            Err(error) => return Err(error),
        }

        Ok(())
    }

    fn parse_debug(&self) -> Result<UciCommand, ParserError> {
        match self.next()? {
            "on" => Ok(true),
            "off" => Ok(false),
            token => Err(InvalidToken(token.to_string())),
        }.map(|value| UciCommand::SetDebug { debug: value })
    }
}

#[cfg(test)]
mod tests {
    use crate::uci::parser::CommandParser;
    use crate::uci::parser::ParserError::{EmptyCommand, EmptyRemaining, EmptyToken, InvalidToken, UnexpectedEndOfCommand, UnknownCommand};
    use crate::uci::UciCommand;
    use crate::uci::UciCommand::{IsReady, PonderHit, Quit, SetDebug, SetOption, SetOptionValue, Stop, Uci, UciNewGame};

    #[test]
    fn general() {
        assert_eq!(CommandParser::new("").parse(), Err(EmptyCommand));
        assert_eq!(CommandParser::new("   ").parse(), Err(EmptyCommand));
        assert_eq!(CommandParser::new("something").parse(), Err(UnknownCommand("something".to_string())));
        assert_eq!(CommandParser::new("something   ").parse(), Err(UnknownCommand("something".to_string())));
        assert_eq!(CommandParser::new("").parse(), Err(EmptyCommand));
    }

    #[test]
    fn debug() {
        assert_eq!(CommandParser::new("debug on").parse(), Ok(SetDebug { debug: true }));
        assert_eq!(CommandParser::new("debug off").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new("debug off something").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new("debug off ").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new(" debug off ").parse(), Ok(SetDebug { debug: false }));
        assert_eq!(CommandParser::new("debug something").parse(), Err(InvalidToken("something".to_string())));
        assert_eq!(CommandParser::new("debug ").parse(), Err(UnexpectedEndOfCommand));
        assert_eq!(CommandParser::new("debug").parse(), Err(UnexpectedEndOfCommand));
    }

    #[test]
    fn setoption() {
        assert_eq!(CommandParser::new("setoption name foo").parse(), Ok(SetOption { name: "foo".to_string() }));
        assert_eq!(CommandParser::new("setoption name foo ").parse(), Ok(SetOption { name: "foo".to_string() }));
        assert_eq!(CommandParser::new(" setoption name foo").parse(), Ok(SetOption { name: "foo".to_string() }));
        assert_eq!(CommandParser::new("setoption name foo something").parse(), Err(InvalidToken("something".to_string())));
        assert_eq!(CommandParser::new("setoption something foo").parse(), Err(InvalidToken("something".to_string())));
        assert_eq!(CommandParser::new("setoption name foo value 1 2 3 ").parse(), Ok(SetOptionValue { name: "foo".to_string(), value: "1 2 3".to_string() }));
        assert_eq!(CommandParser::new("setoption name foo value  ").parse(), Err(EmptyRemaining));
        assert_eq!(CommandParser::new("setoption   ").parse(), Err(EmptyToken));
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
