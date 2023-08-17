use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use marvk_chess_uci::{UciEngine, ProtectionMessage, UciCommand, UciTx};
use SearchMessage::{UciGo, UciPositionFrom, UciUciNewGame};
use UciCommand::{IsReady, PonderHit, PositionFrom, Quit, Register, RegisterLater, SetDebug, SetOption, SetOptionValue, Stop, Uci, UciNewGame};
use UciCommand::Go as GoCommand;

use crate::engine::heuristic::simple::SimpleHeuristic;
use crate::engine::move_order::MvvLvaMoveOrder;
use crate::engine::search::{EngineOptions, Search, SearchMessage};
use crate::engine::search::SearchMessage::{UciDebug, UciPonderHit, UciQuit, UciStop};

mod heuristic;
mod move_order;
mod zobrist_history;
mod metrics;
mod search;
mod table;

pub struct Engine<T: UciTx + Send + Sync + 'static> {
    uci_tx: Arc<T>,
    debug: bool,
    search_tx: Sender<SearchMessage>,
    search_handle: Option<JoinHandle<()>>,
}

impl<T: UciTx + Send + Sync + 'static> Engine<T> {
    pub fn new(uci_tx: Arc<T>, debug: bool) -> Self {
        let (search_tx, search_rx) = channel();
        let search_handle = Self::start_search_thread(search_rx, uci_tx.clone(), debug);

        Self { uci_tx, debug, search_tx, search_handle: Some(search_handle) }
    }

    fn start_search_thread(search_rx: Receiver<SearchMessage>, uci_tx: Arc<T>, debug: bool) -> JoinHandle<()> {
        thread::spawn(move || {
            Search::new(uci_tx, search_rx, SimpleHeuristic, MvvLvaMoveOrder, EngineOptions { debug, ..EngineOptions::default() }).idle();
        })
    }
}

impl<T: UciTx + Send + Sync + 'static> UciEngine for Engine<T> {
    #[allow(unused_variables)]
    #[allow(clippy::unwrap_used)]
    fn accept(&mut self, command: UciCommand) {
        match command {
            Uci => {
                self.uci_tx.id_name("Inkayaku");
                self.uci_tx.id_author("Marvin Kuhnke (see https://github.com/marvk/rust-chess)");
                self.uci_tx.uci_ok();
            }
            SetDebug { debug } => {
                self.debug = debug;
                self.search_tx.send(UciDebug(debug)).unwrap();
            }
            IsReady => {
                self.uci_tx.ready_ok();
            }
            SetOption { name } => {
                todo!()
            }
            SetOptionValue { name, value } => {
                todo!()
            }
            RegisterLater => {}
            Register { .. } => {
                self.uci_tx.registration(ProtectionMessage::CHECKING);
                self.uci_tx.registration(ProtectionMessage::OK);
            }
            UciNewGame => {
                self.search_tx.send(UciUciNewGame).unwrap();
            }
            PositionFrom { fen, moves } => {
                self.search_tx.send(UciPositionFrom(fen, moves)).unwrap();
            }
            GoCommand { go } => {
                self.search_tx.send(UciGo(go)).unwrap();
            }
            Stop => {
                self.search_tx.send(UciStop).unwrap();
            }
            PonderHit => {
                self.search_tx.send(UciPonderHit).unwrap();
            }
            Quit => {
                self.search_tx.send(UciQuit).unwrap();
                self.search_handle.take().unwrap().join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::mpsc::channel;

    use marvk_chess_core::fen::Fen;
    use marvk_chess_uci::{UciEngine, Go, Score, UciCommand, UciMove, UciTxCommand};
    use marvk_chess_uci::command::CommandUciTx;

    use crate::engine::Engine;

    #[test]
    fn test_threefold() {
        let (tx, rx) = channel();
        let mut engine = Engine::new(Arc::new(CommandUciTx::new(tx)), false);
        engine.accept(UciCommand::UciNewGame);

        let wait_for_best_move = || {
            rx.iter().filter(|m| matches!(m, UciTxCommand::BestMove {..})).take(1).last().unwrap()
        };

        let moves = vec!["b1c3", "b8c6", "d2d4", "d7d5", "e2e4", "e7e6", "e4e5", "f8b4", "g1e2", "g8e7", "c1f4", "e8g8", "d1d3", "f7f6", "e5f6", "f8f6", "e1c1", "b4d6", "f4d6", "c7d6", "d3e3", "e6e5", "d4e5", "d6e5", "c3e4", "f6f7", "c1b1", "d8c7", "e4g5", "f7f8", "e2c3", "c6d4", "f1d3", "h7h6", "g5f3", "d4f3", "g2f3", "d5d4", "c3b5", "c7c2", "b1c2", "d4e3", "f2e3", "f8f3", "b5c7", "a8b8", "d3c4", "g8h7", "e3e4", "c8g4", "c4e6", "g4e6", "c7e6", "b8c8", "c2b1", "f3f6", "e6d8", "c8c7", "d1d3", "a7a6", "h1c1", "c7c1", "b1c1", "f6f1", "c1c2", "f1f4", "d3d7", "e7c6", "d8e6", "f4e4", "d7g7", "h7h8", "g7b7", "e4h4", "b7c7", "h4h2", "c2c3", "c6d4", "e6d4", "e5d4", "c3d4", "h2b2", "a2a3", "b2a2", "c7c8", "h8g7", "c8c7", "g7f8", "c7h7", "a2a3", "h7h6", "f8e7", "h6b6", "a3a2", "d4d3"].into_iter().map(|s| UciMove::parse(s).unwrap()).collect::<Vec<_>>();
        let go = Go {
            depth: Some(8),
            ..Go::default()
        };
        engine.accept(UciCommand::PositionFrom { fen: Fen::default(), moves });
        engine.accept(UciCommand::Go { go });
        wait_for_best_move();

        let moves = vec!["b1c3", "b8c6", "d2d4", "d7d5", "e2e4", "e7e6", "e4e5", "f8b4", "g1e2", "g8e7", "c1f4", "e8g8", "d1d3", "f7f6", "e5f6", "f8f6", "e1c1", "b4d6", "f4d6", "c7d6", "d3e3", "e6e5", "d4e5", "d6e5", "c3e4", "f6f7", "c1b1", "d8c7", "e4g5", "f7f8", "e2c3", "c6d4", "f1d3", "h7h6", "g5f3", "d4f3", "g2f3", "d5d4", "c3b5", "c7c2", "b1c2", "d4e3", "f2e3", "f8f3", "b5c7", "a8b8", "d3c4", "g8h7", "e3e4", "c8g4", "c4e6", "g4e6", "c7e6", "b8c8", "c2b1", "f3f6", "e6d8", "c8c7", "d1d3", "a7a6", "h1c1", "c7c1", "b1c1", "f6f1", "c1c2", "f1f4", "d3d7", "e7c6", "d8e6", "f4e4", "d7g7", "h7h8", "g7b7", "e4h4", "b7c7", "h4h2", "c2c3", "c6d4", "e6d4", "e5d4", "c3d4", "h2b2", "a2a3", "b2a2", "c7c8", "h8g7", "c8c7", "g7f8", "c7h7", "a2a3", "h7h6", "f8e7", "h6b6", "a3a2", "d4d3", "a2a5", "d3d4"].into_iter().map(|s| UciMove::parse(s).unwrap()).collect::<Vec<_>>();
        let go = Go {
            depth: Some(10),
            ..Go::default()
        };
        engine.accept(UciCommand::PositionFrom { fen: Fen::default(), moves });
        engine.accept(UciCommand::Go { go });
        wait_for_best_move();

        let moves = vec!["b1c3", "b8c6", "d2d4", "d7d5", "e2e4", "e7e6", "e4e5", "f8b4", "g1e2", "g8e7", "c1f4", "e8g8", "d1d3", "f7f6", "e5f6", "f8f6", "e1c1", "b4d6", "f4d6", "c7d6", "d3e3", "e6e5", "d4e5", "d6e5", "c3e4", "f6f7", "c1b1", "d8c7", "e4g5", "f7f8", "e2c3", "c6d4", "f1d3", "h7h6", "g5f3", "d4f3", "g2f3", "d5d4", "c3b5", "c7c2", "b1c2", "d4e3", "f2e3", "f8f3", "b5c7", "a8b8", "d3c4", "g8h7", "e3e4", "c8g4", "c4e6", "g4e6", "c7e6", "b8c8", "c2b1", "f3f6", "e6d8", "c8c7", "d1d3", "a7a6", "h1c1", "c7c1", "b1c1", "f6f1", "c1c2", "f1f4", "d3d7", "e7c6", "d8e6", "f4e4", "d7g7", "h7h8", "g7b7", "e4h4", "b7c7", "h4h2", "c2c3", "c6d4", "e6d4", "e5d4", "c3d4", "h2b2", "a2a3", "b2a2", "c7c8", "h8g7", "c8c7", "g7f8", "c7h7", "a2a3", "h7h6", "f8e7", "h6b6", "a3a2", "d4d3", "a2a5", "d3d4", "a5a2", "d4d3"].into_iter().map(|s| UciMove::parse(s).unwrap()).collect::<Vec<_>>();
        let go = Go {
            depth: Some(9),
            ..Go::default()
        };
        engine.accept(UciCommand::PositionFrom { fen: Fen::default(), moves });
        engine.accept(UciCommand::Go { go });
        wait_for_best_move();

        let moves = vec!["b1c3", "b8c6", "d2d4", "d7d5", "e2e4", "e7e6", "e4e5", "f8b4", "g1e2", "g8e7", "c1f4", "e8g8", "d1d3", "f7f6", "e5f6", "f8f6", "e1c1", "b4d6", "f4d6", "c7d6", "d3e3", "e6e5", "d4e5", "d6e5", "c3e4", "f6f7", "c1b1", "d8c7", "e4g5", "f7f8", "e2c3", "c6d4", "f1d3", "h7h6", "g5f3", "d4f3", "g2f3", "d5d4", "c3b5", "c7c2", "b1c2", "d4e3", "f2e3", "f8f3", "b5c7", "a8b8", "d3c4", "g8h7", "e3e4", "c8g4", "c4e6", "g4e6", "c7e6", "b8c8", "c2b1", "f3f6", "e6d8", "c8c7", "d1d3", "a7a6", "h1c1", "c7c1", "b1c1", "f6f1", "c1c2", "f1f4", "d3d7", "e7c6", "d8e6", "f4e4", "d7g7", "h7h8", "g7b7", "e4h4", "b7c7", "h4h2", "c2c3", "c6d4", "e6d4", "e5d4", "c3d4", "h2b2", "a2a3", "b2a2", "c7c8", "h8g7", "c8c7", "g7f8", "c7h7", "a2a3", "h7h6", "f8e7", "h6b6", "a3a2", "d4d3", "a2a5", "d3d4", "a5a2", "d4d3", "a2a5", "d3d4"].into_iter().map(|s| UciMove::parse(s).unwrap()).collect::<Vec<_>>();
        let go = Go {
            depth: Some(10),
            search_moves: vec![UciMove::parse("a5a2").unwrap()],
            ..Go::default()
        };
        engine.accept(UciCommand::PositionFrom { fen: Fen::default(), moves });
        engine.accept(UciCommand::Go { go });
        while let Ok(c) = rx.recv() {
            dbg!(&c);

            if let UciTxCommand::BestMove { .. } = c {
                break;
            }
        }
    }

    #[test]
    fn test_threefold_1() {
        let fen = Fen::from_str("5rk1/5r2/p7/2pNp1q1/2P1P2p/1P3P1P/P4RP1/5RK1 w - - 0 28").unwrap();
        let moves = vec![
            "d5b6", "g5e3", "b6d5", "e3g5",
            "d5b6", "g5e3", "b6d5",
        ];
        let move_to_draw = "e3g5";
        _test_threefold(moves, fen, move_to_draw);
    }

    #[test]
    fn test_threefold_2() {
        let fen = Fen::default();
        let moves = vec!["e2e4", "b8c6", "g1f3", "g8f6", "e4e5", "f6d5", "d2d4", "d7d6", "c2c4", "d5b6", "b1c3", "c8g4", "c1f4", "e7e6", "d1d3", "g4f3", "g2f3", "d6e5", "d4e5", "d8d4", "d3d4", "c6d4", "e1c1", "c7c5", "f4g3", "e8e7", "f3f4", "d4f5", "f1d3", "f5g3", "h2g3", "f7f6", "c1d2", "f6e5", "f4e5", "b6d7", "f2f4", "d7b6", "d2e3", "e7f7", "g3g4", "f8e7", "a2a4", "a7a5", "d1e1", "a8d8", "g4g5", "d8a8", "h1h3", "h7h6", "h3f3", "h6g5", "f4g5", "f7e8", "f3g3", "h8h2", "e1e2", "h2e2", "d3e2", "e8d7", "g3g4", "a8h8", "e2f3", "h8h3", "b2b3", "d7c7", "c3b5", "c7d7", "g4g2", "d7c8", "e3f4", "h3h4", "g2g4", "h4h3", "g4g2", "h3h4", "g2g4", "h4h3"];
        let move_to_draw = "g4g2";
        _test_threefold(moves, fen, move_to_draw);
    }

    #[test]
    fn test_threefold_3() {
        let fen = Fen::from_str("5r1k/5r2/p7/2pNp1q1/2P1P2p/1P3P1P/P4RP1/5RK1 b - - 0 28").unwrap();
        let moves = vec![
            "h8g8",
            "d5b6", "g5e3", "b6d5", "e3g5",
            "d5b6", "g5e3",
        ];
        let move_to_draw = "b6d5";
        _test_threefold(moves, fen, move_to_draw);
    }

    fn _test_threefold(moves: Vec<&str>, fen: Fen, move_to_draw: &str) {
        let (tx, rx) = channel();
        let mut engine = Engine::new(Arc::new(CommandUciTx::new(tx)), false);

        engine.accept(UciCommand::UciNewGame);
        let uci_moves = moves.into_iter().map(|s| UciMove::parse(s).unwrap()).collect();
        engine.accept(UciCommand::PositionFrom { fen, moves: uci_moves });
        engine.accept(UciCommand::Go { go: Go { depth: Some(5), search_moves: vec![UciMove::parse(move_to_draw).unwrap()], ..Go::default() } });

        let mut commands = Vec::new();

        while let Ok(command) = rx.recv() {
            commands.push(command);
            if let UciTxCommand::BestMove { .. } = commands.last().unwrap() {
                break;
            }
        }

        if let Some(UciTxCommand::Info { info }) = commands.into_iter().filter(|c| matches!(c, UciTxCommand::Info {..})).last() {
            assert_eq!(info.score, Some(Score::Centipawn { score: 0 }));
        } else {
            assert_eq!(true, false, "No score was send");
        }
    }
}
