use std::fmt::Display;
use std::io::Error as IoError;
use std::sync::Mutex;

use crate::uci::{CurrentLine, Info, ProtectionMessage, Score, UciCommand, UciMove, UciTx};
use crate::uci::console::ConsoleUciRxError::{CommandParseError, SystemError};
use crate::uci::parser::{CommandParser, ParserError};

#[derive(Debug)]
pub enum ConsoleUciRxError {
    SystemError(IoError),
    CommandParseError(ParserError),
}

pub struct ConsoleUciTx<FConsumer: Fn(&str), FDebugConsumer: Fn(&str)> {
    consumer: FConsumer,
    debug_consumer: FDebugConsumer,
    debug: Mutex<bool>,
}

impl<FConsumer: Fn(&str), FDebugConsumer: Fn(&str)> ConsoleUciTx<FConsumer, FDebugConsumer> {
    pub const fn new(consumer: FConsumer, error_consumer: FDebugConsumer, debug: bool) -> Self {
        Self { consumer, debug_consumer: error_consumer, debug: Mutex::new(debug) }
    }

    #[allow(clippy::unwrap_used)]
    pub fn set_debug(&self, debug: bool) {
        *self.debug.lock().unwrap() = debug;
    }

    fn tx(&self, message: &str) {
        (self.consumer)(message);
    }

    #[allow(clippy::unwrap_used)]
    fn tx_debug(&self, message: &str) {
        if *self.debug.lock().unwrap() {
            (self.debug_consumer)(message);
        }
    }

    fn tx_options(&self, name: &str, the_type: &str, remainder: &str) {
        self.tx(format!("option name {} type {} {}", name, the_type, remainder).trim());
    }
}

impl<FConsumer: Fn(&str), FDebugConsumer: Fn(&str)> UciTx for ConsoleUciTx<FConsumer, FDebugConsumer> {
    fn id_name(&self, name: &str) {
        assert!(!name.is_empty());

        self.tx(&format!("id name {}", name));
    }

    fn id_author(&self, author: &str) {
        assert!(!author.is_empty());

        self.tx(&format!("id author {}", author));
    }

    fn uci_ok(&self) {
        self.tx("uciok");
    }

    fn ready_ok(&self) {
        self.tx("readyok");
    }

    fn best_move(&self, best_move: Option<UciMove>, ponder_move: Option<UciMove>) {
        let move_string = best_move.map_or_else(|| "0000".to_string(), |mv| format!("{}", mv));
        let ponder_string = ponder_move.map_or_else(String::new, |ponder_mv| format!(" ponder {}", ponder_mv));

        self.tx(&format!("bestmove {}{}", move_string, ponder_string));
    }

    fn copy_protection(&self, copy_protection: ProtectionMessage) {
        self.tx(&format!("copyprotection {}", copy_protection));
    }

    fn registration(&self, registration: ProtectionMessage) {
        self.tx(&format!("registration {}", registration));
    }

    fn info(&self, info: &Info) {
        fn append_maybe<T: Display>(accumulator: &mut String, key: &str, value: Option<T>) {
            if let Some(value) = value {
                accumulator.push_str(&format!(" {} {}", key, value));
            }
        }

        fn move_array_to_string(uci_moves: &[UciMove]) -> String {
            uci_moves.iter().map(|m| format!("{}", m)).collect::<Vec<_>>().join(" ")
        }

        fn score_to_string(score: Score) -> String {
            match score {
                Score::Mate { mate_in } => format!("mate {}", mate_in),
                Score::Centipawn { score: centipawn_value } => format!("cp {}", centipawn_value),
                Score::CentipawnBounded { score: centipawn_value, bound } => format!("cp {} {}", centipawn_value, bound),
            }
        }

        fn current_line_to_string(current_line: &CurrentLine) -> String {
            format!("{} {}", current_line.cpu_number, move_array_to_string(&current_line.line))
        }

        let mut msg = "info".to_string();

        append_maybe(&mut msg, "depth", info.depth);
        append_maybe(&mut msg, "seldepth", info.selective_depth);
        append_maybe(&mut msg, "time", info.time.map(|d| d.as_millis()));
        append_maybe(&mut msg, "nodes", info.nodes);
        append_maybe(&mut msg, "pv", info.principal_variation.as_deref().map(move_array_to_string));
        append_maybe(&mut msg, "multipv", info.multi_pv);
        append_maybe(&mut msg, "score", info.score.map(score_to_string));
        append_maybe(&mut msg, "currmove", info.current_move.as_ref());
        append_maybe(&mut msg, "currmovenumber", info.current_move_number);
        append_maybe(&mut msg, "hashfull", info.hash_full);
        append_maybe(&mut msg, "nps", info.nps);
        append_maybe(&mut msg, "tbhits", info.table_hits);
        append_maybe(&mut msg, "sbhits", info.shredder_table_hits);
        append_maybe(&mut msg, "cpuload", info.cpu_load);
        append_maybe(&mut msg, "refutation", info.refutation.as_deref().map(move_array_to_string));
        append_maybe(&mut msg, "currline", info.current_line.as_ref().map(current_line_to_string));
        append_maybe(&mut msg, "string", info.string.as_ref());

        self.tx(&msg);
    }

    fn option_check(&self, name: &str, default: bool) {
        self.tx_options(name, "check", &format!("default {}", default));
    }

    fn option_spin(&self, name: &str, default: i32, min: i32, max: i32) {
        self.tx_options(name, "spin", &format!("default {} min {} max {}", default, min, max));
    }

    fn option_combo(&self, name: &str, default: &str, vars: &[&str]) {
        let mut vars_string = String::new();

        for &var in vars {
            vars_string.push_str(" var ");
            vars_string.push_str(var);
        }

        self.tx_options(name, "combo", &format!("default {}{}", default, vars_string));
    }

    fn option_button(&self, name: &str) {
        self.tx_options(name, "button", "");
    }

    fn option_string(&self, name: &str, default: &str) {
        self.tx_options(name, "string", &format!("default {}", default));
    }

    fn debug(&self, message: &str) {
        self.tx_debug(message);
    }
}

pub struct ConsoleUciRx<FRead: Fn() -> Result<String, IoError>, FOnCommand: Fn(Result<UciCommand, ConsoleUciRxError>)> {
    read: FRead,
    on_command: FOnCommand,
}

impl<FRead: Fn() -> Result<String, IoError>, FOnCommand: Fn(Result<UciCommand, ConsoleUciRxError>)> ConsoleUciRx<FRead, FOnCommand> {
    pub const fn new(read: FRead, on_command: FOnCommand) -> Self {
        Self { read, on_command }
    }

    pub fn start(&self) {
        loop {
            let command = self.read_next_command();
            let is_quit = matches!(command, Ok(UciCommand::Quit));
            (self.on_command)(command);

            if is_quit {
                return;
            }
        }
    }

    fn read_next_command(&self) -> Result<UciCommand, ConsoleUciRxError> {
        (self.read)().map_err(SystemError).and_then(|raw| {
            CommandParser::new(&raw).parse().map_err(CommandParseError)
        })
    }
}


// #[cfg(test)]
// mod tests {
//     use std::io::stdin;
//     use std::sync::{Arc, Mutex};
//     use std::time::Duration;
//
//     use inkayaku_core::constants::Piece;
//     use inkayaku_core::constants::Square;
//     use inkayaku_core::fen::Fen;
//
//     use crate::uci::{Bound, Engine, Go, Info, ProtectionMessage, Score, UciCommand, UciMove, UciTx};
//     use crate::uci::console::{ConsoleUciRx, ConsoleUciTx};
//
//     struct TestEngine;
//
//     impl Engine for TestEngine {
//         fn accept(&self, command: UciCommand) {
//             println!("{:?}", command);
//         }
//     }
//
//     #[test]
//     #[ignore]
//     fn test() {
//         let read = || {
//             let mut result = String::new();
//             stdin().read_line(&mut result).map(|_| result)
//         };
//
//         let engine = TestEngine {};
//         let on_command = move |command_result| {
//             if let Ok(command) = command_result {
//                 engine.accept(command);
//             }
//         };
//
//         ConsoleUciRx::new(read, on_command).start();
//     }
//
//     struct MessageBuffer {
//         messages: Vec<String>,
//     }
//
//     impl<'a> MessageBuffer {
//         fn append(&mut self, msg: String) {
//             self.messages.push(msg);
//         }
//     }
//
//     #[test]
//     fn id_name() {
//         run_test(|sut| sut.id_name("marv"), "id name marv")
//     }
//
//     #[test]
//     #[should_panic]
//     fn id_name_panic() {
//         run_test(|sut| sut.id_name(""), "")
//     }
//
//     #[test]
//     fn id_author() {
//         run_test(|sut| sut.id_author("marv"), "id author marv")
//     }
//
//     #[test]
//     fn uci_ok() {
//         run_test(|sut| sut.uci_ok(), "uciok")
//     }
//
//     #[test]
//     fn ready_ok() {
//         run_test(|sut| sut.ready_ok(), "readyok")
//     }
//
//     #[test]
//     fn best_move() {
//         let m = UciMove::new(Square::A1, Square::A2);
//
//         run_test(|sut| { sut.best_move(&m) }, "bestmove a1a2")
//     }
//
//     #[test]
//     fn best_move_promotion() {
//         let m = UciMove::new_with_promotion(Square::A1, Square::A2, Piece::QUEEN);
//
//         run_test(|sut| { sut.best_move(&m) }, "bestmove a1a2q")
//     }
//
//     #[test]
//     fn best_move_ponder() {
//         let m = UciMove::new(Square::A1, Square::A2);
//         let p = UciMove::new(Square::A5, Square::A6);
//
//         run_test(|sut| { sut.best_move_with_ponder(&m, &p) }, "bestmove a1a2 ponder a5a6")
//     }
//
//     #[test]
//     fn best_move_ponder_promotion() {
//         let m = UciMove::new_with_promotion(Square::A1, Square::A2, Piece::QUEEN);
//         let p = UciMove::new_with_promotion(Square::A5, Square::A6, Piece::QUEEN);
//
//         run_test(|sut| { sut.best_move_with_ponder(&m, &p) }, "bestmove a1a2q ponder a5a6q")
//     }
//
//     #[test]
//     fn copy_protection() {
//         run_test(|sut| sut.copy_protection(ProtectionMessage::OK), "copyprotection ok")
//     }
//
//     #[test]
//     fn registration() {
//         run_test(|sut| sut.registration(ProtectionMessage::ERROR), "registration error")
//     }
//
//     #[test]
//     fn info_empty() {
//         let info = Info::EMPTY;
//
//         run_test(|sut| sut.info(&info), "info")
//     }
//
//     #[test]
//     fn info_current_move() {
//         let info = Info {
//             current_move: Some(UciMove::new(Square::A1, Square::A2)),
//             ..Info::EMPTY
//         };
//
//         run_test(|sut| sut.info(&info), "info currmove a1a2")
//     }
//
//     #[test]
//     fn info_all() {
//         let principal_variation = [UciMove::new(Square::A1, Square::A2), UciMove::new(Square::A3, Square::A4)];
//         let refutation = [UciMove::new(Square::D1, Square::D2), UciMove::new(Square::C3, Square::C4)];
//         let current_line = [UciMove::new(Square::H1, Square::H2), UciMove::new(Square::B3, Square::B4)];
//         let info = Info::new(
//             20,
//             10,
//             Duration::from_micros(21234584),
//             45000000,
//             &principal_variation,
//             1,
//             Score::CentipawnBounded { score: 200, bound: Bound::LOWER },
//             UciMove::new_with_promotion(Square::H8, Square::H7, Piece::QUEEN),
//             24,
//             80,
//             200000000,
//             213333,
//             2040,
//             99,
//             "hi it's info",
//             &refutation,
//             1,
//             &current_line,
//         );
//
//         run_test(|sut| sut.info(&info), "info depth 20 seldepth 10 time 21234 nodes 45000000 pv a1a2 a3a4 multipv 1 score cp 200 lowerbound currmove h8h7q currmovenumber 24 hashfull 80 nps 200000000 tbhits 213333 sbhits 2040 cpuload 99 refutation d1d2 c3c4 currline 1 h1h2 b3b4 string hi it's info")
//     }
//
//     #[test]
//     fn option_button() {
//         run_test(|sut| sut.option_button("Clear Hash"), "option name Clear Hash type button")
//     }
//
//     #[test]
//     fn option_check() {
//         run_test(|sut| sut.option_check("Nullmove", true), "option name Nullmove type check default true")
//     }
//
//     #[test]
//     fn option_spin() {
//         run_test(|sut| sut.option_spin("Selectivity", 2, 0, 4), "option name Selectivity type spin default 2 min 0 max 4")
//     }
//
//     #[test]
//     fn option_combo() {
//         run_test(|sut| sut.option_combo("Style", "Normal", &["Solid", "Normal", "Risky"]), "option name Style type combo default Normal var Solid var Normal var Risky")
//     }
//
//     #[test]
//     fn option_string() {
//         run_test(|sut| sut.option_string("NalimovPath", "c:\\"), "option name NalimovPath type string default c:\\")
//     }
//
//
//     fn run_test<C: , F: Fn(&ConsoleUciTx<dyn Fn(&str)>)>(run_sut: F, expected: &str) {
//         let buffer = Arc::new(Mutex::new(MessageBuffer { messages: Vec::new() }));
//         let closure_buffer = Arc::clone(&buffer);
//
//         let sut = ConsoleUciTx {
//             consumer: Box::new(move |str: &str| {
//                 closure_buffer.lock().unwrap().append(str.to_string());
//             })
//         };
//
//         run_sut(&sut);
//
//         let vec = buffer.lock().unwrap().messages.clone();
//         assert_eq!(vec, &[expected])
//     }
// }
