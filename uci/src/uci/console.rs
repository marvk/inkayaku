use std::collections::HashMap;
use std::fmt::Display;
use std::io::Error as IoError;
use std::str::Split;

use crate::uci;
use crate::uci::{CurrentLine, Engine, Info, ProtectionMessage, Score, UciCommand, UciMove, UciTx};

struct ConsoleUciTx {
    consumer: Box<dyn Fn(&str)>,
}

impl ConsoleUciTx {
    pub fn new<F>(consumer: F) -> Self
        where F: Fn(&str),
              F: 'static
    {
        Self { consumer: Box::new(consumer) }
    }

    fn tx(&self, message: &str) {
        (self.consumer)(message)
    }
}

impl UciTx for ConsoleUciTx {
    fn id_name(&self, name: &str) {
        if name.is_empty() {
            panic!()
        }

        self.tx(&format!("id name {}", name))
    }

    fn id_author(&self, author: &str) {
        if author.is_empty() {
            panic!()
        }

        self.tx(&format!("id author {}", author))
    }

    fn uci_ok(&self) {
        self.tx("uciok")
    }

    fn ready_ok(&self) {
        self.tx("readyok")
    }

    fn best_move(&self, uci_move: &UciMove) {
        self.tx(&format!("bestmove {}", uci_move))
    }

    fn best_move_with_ponder(&self, uci_move: &UciMove, ponder_uci_move: &UciMove) {
        self.tx(&format!("bestmove {} ponder {}", uci_move, ponder_uci_move))
    }

    fn copy_protection(&self, copy_protection: ProtectionMessage) {
        self.tx(&format!("copyprotection {}", copy_protection))
    }

    fn registration(&self, registration: ProtectionMessage) {
        self.tx(&format!("registration {}", registration))
    }

    fn info(&self, info: &Info) {
        let mut msg = "info".to_string();

        fn append_maybe<T: Display>(accumulator: &mut String, key: &str, value: Option<T>) {
            if value.is_some() {
                accumulator.push_str(&format!(" {} {}", key, value.unwrap()))
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
            format!("{} {}", current_line.cpu_number, move_array_to_string(current_line.line))
        }

        append_maybe(&mut msg, "depth", info.depth);
        append_maybe(&mut msg, "seldepth", info.selective_depth);
        append_maybe(&mut msg, "time", info.time.map(|d| d.as_millis()));
        append_maybe(&mut msg, "nodes", info.nodes);
        append_maybe(&mut msg, "pv", info.principal_variation.map(move_array_to_string));
        append_maybe(&mut msg, "multipv", info.multi_pv);
        append_maybe(&mut msg, "score", info.score.map(score_to_string));
        append_maybe(&mut msg, "currmove", info.current_move.as_ref());
        append_maybe(&mut msg, "currmovenumber", info.current_move_number);
        append_maybe(&mut msg, "hashfull", info.hash_full);
        append_maybe(&mut msg, "nps", info.nps);
        append_maybe(&mut msg, "tbhits", info.table_hits);
        append_maybe(&mut msg, "sbhits", info.shredder_table_hits);
        append_maybe(&mut msg, "cpuload", info.cpu_load);
        append_maybe(&mut msg, "refutation", info.refutation.map(move_array_to_string));
        append_maybe(&mut msg, "currline", info.current_line.as_ref().map(current_line_to_string));
        append_maybe(&mut msg, "string", info.string);

        self.tx(&msg)
    }
}



struct ConsoleUciRx<E: Engine> {
    engine: E,
    read: Box<dyn Fn() -> Result<String, IoError>>,
}

impl<E: Engine> ConsoleUciRx<E> {
    pub fn new<F>(engine: E, read: F) -> Self
        where F: Fn() -> Result<String, IoError>,
              F: 'static,
    {
        Self { engine, read: Box::new(read) }
    }


    pub fn start(&self) {
        loop {
            if let Ok(raw_command) = (self.read)() {
                todo!()
                // if let Some(command) = CommandParser::parse(&raw_command) {
                //     self.engine.accept(command);
                // }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io::stdin;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use marvk_chess_core::constants::piece::Piece;
    use marvk_chess_core::constants::square::Square;
    use marvk_chess_core::fen::Fen;

    use crate::uci::{Bound, Engine, Go, Info, ProtectionMessage, Score, UciCommand, UciMove, UciTx};
    use crate::uci::console::{ConsoleUciRx, ConsoleUciTx};

    struct TestEngine;

    impl Engine for TestEngine {
        fn accept(&self, _: UciCommand) {}
    }

    #[test]
    fn test() {
        let read = || {
            let mut result = String::new();
            stdin().read_line(&mut result).map(|_| result)
        };

        ConsoleUciRx::new(TestEngine {}, read).start();
    }

    struct MessageBuffer {
        messages: Vec<String>,
    }

    impl<'a> MessageBuffer {
        fn append(&mut self, msg: String) {
            self.messages.push(msg);
        }
    }

    #[test]
    fn id_name() {
        run_test(|sut| sut.id_name("marv"), "id name marv")
    }

    #[test]
    #[should_panic]
    fn id_name_panic() {
        run_test(|sut| sut.id_name(""), "")
    }

    #[test]
    fn id_author() {
        run_test(|sut| sut.id_author("marv"), "id author marv")
    }

    #[test]
    fn uci_ok() {
        run_test(|sut| sut.uci_ok(), "uciok")
    }

    #[test]
    fn ready_ok() {
        run_test(|sut| sut.ready_ok(), "readyok")
    }

    #[test]
    fn best_move() {
        let m = UciMove::new(Square::A1, Square::A2);

        run_test(|sut| { sut.best_move(&m) }, "bestmove a1a2")
    }

    #[test]
    fn best_move_promotion() {
        let m = UciMove::new_with_promotion(Square::A1, Square::A2, Piece::QUEEN);

        run_test(|sut| { sut.best_move(&m) }, "bestmove a1a2q")
    }

    #[test]
    fn best_move_ponder() {
        let m = UciMove::new(Square::A1, Square::A2);
        let p = UciMove::new(Square::A5, Square::A6);

        run_test(|sut| { sut.best_move_with_ponder(&m, &p) }, "bestmove a1a2 ponder a5a6")
    }

    #[test]
    fn best_move_ponder_promotion() {
        let m = UciMove::new_with_promotion(Square::A1, Square::A2, Piece::QUEEN);
        let p = UciMove::new_with_promotion(Square::A5, Square::A6, Piece::QUEEN);

        run_test(|sut| { sut.best_move_with_ponder(&m, &p) }, "bestmove a1a2q ponder a5a6q")
    }

    #[test]
    fn copy_protection() {
        run_test(|sut| sut.copy_protection(ProtectionMessage::OK), "copyprotection ok")
    }

    #[test]
    fn registration() {
        run_test(|sut| sut.registration(ProtectionMessage::ERROR), "registration error")
    }

    #[test]
    fn info_empty() {
        let info = Info::EMPTY;

        run_test(|sut| sut.info(&info), "info")
    }

    #[test]
    fn info_current_move() {
        let info = Info {
            current_move: Some(UciMove::new(Square::A1, Square::A2)),
            ..Info::EMPTY
        };

        run_test(|sut| sut.info(&info), "info currmove a1a2")
    }

    #[test]
    fn info_all() {
        let principal_variation = [UciMove::new(Square::A1, Square::A2), UciMove::new(Square::A3, Square::A4)];
        let refutation = [UciMove::new(Square::D1, Square::D2), UciMove::new(Square::C3, Square::C4)];
        let current_line = [UciMove::new(Square::H1, Square::H2), UciMove::new(Square::B3, Square::B4)];
        let info = Info::new(
            20,
            10,
            Duration::from_micros(21234584),
            45000000,
            &principal_variation,
            1,
            Score::CentipawnBounded { score: 200, bound: Bound::LOWER },
            UciMove::new_with_promotion(Square::H8, Square::H7, Piece::QUEEN),
            24,
            80,
            200000000,
            213333,
            2040,
            99,
            "hi it's info",
            &refutation,
            1,
            &current_line,
        );

        run_test(|sut| sut.info(&info), "info depth 20 seldepth 10 time 21234 nodes 45000000 pv a1a2 a3a4 multipv 1 score cp 200 lowerbound currmove h8h7q currmovenumber 24 hashfull 80 nps 200000000 tbhits 213333 sbhits 2040 cpuload 99 refutation d1d2 c3c4 currline 1 h1h2 b3b4 string hi it's info")
    }

    #[test]
    fn main() {
        let tx = ConsoleUciTx::new(|str| println!("{}", str));

        tx.id_author("Marv");
    }


    fn run_test<F: Fn(&ConsoleUciTx)>(run_sut: F, expected: &str) {
        let buffer = Arc::new(Mutex::new(MessageBuffer { messages: Vec::new() }));
        let closure_buffer = Arc::clone(&buffer);

        let sut = ConsoleUciTx {
            consumer: Box::new(move |str| {
                closure_buffer.lock().unwrap().append(str.to_string());
            })
        };

        run_sut(&sut);

        let vec = buffer.lock().unwrap().messages.clone();
        assert_eq!(vec, &[expected])
    }
}
