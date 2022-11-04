use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use rand::prelude::SliceRandom;
use rand::thread_rng;

use marvk_chess_board::board::{Bitboard, Move};
use marvk_chess_core::fen::{Fen, FEN_STARTPOS};
use marvk_chess_uci::uci::{Engine, Go, ProtectionMessage, UciCommand, UciMove, UciTx};
use UciCommand::*;
use UciCommand::Go as GoCommand;

pub struct Inkayaku<T: UciTx + Send + Sync+ 'static> {
    tx: Arc<T>,
    fen: Fen,
    search: SearchThreadHolder<T>,
}

impl<T: UciTx + Send + Sync + 'static> Inkayaku<T> {
    pub fn new(tx: T) -> Self {
        let tx_arc = Arc::new(tx);
        Self { tx: tx_arc.clone(), fen: FEN_STARTPOS.clone(), search: SearchThreadHolder::new(tx_arc) }
    }

    fn set_from_position(&mut self, fen: Fen, moves: &[String]) {
        let mut board = Bitboard::new(&fen);

        for x in moves {
            if let Err(error) = board.make_san(x) {
                println!("ERROR: {:?}", error);
                return;
            }
        }

        self.fen = board.fen();
    }

    fn start_new_search(&mut self, go: Go) {
        self.search.start(self.fen.clone(), go);
    }
}

impl<T: UciTx + Send + Sync + 'static> Engine for Inkayaku<T> {
    fn accept(&mut self, command: UciCommand) {
        match command {
            Uci => {
                self.tx.id_name("Inkayaku");
                self.tx.id_author("Marvin Kuhnke, check https://github.com/marvk/rust-chess");
                self.tx.uci_ok();
            }
            SetDebug { debug } => {
                todo!()
            }
            IsReady => {
                self.tx.ready_ok();
            }
            SetOption { name } => {
                todo!()
            }
            SetOptionValue { name, value } => {
                todo!()
            }
            RegisterLater => {}
            Register { .. } => {
                self.tx.registration(ProtectionMessage::CHECKING);
                self.tx.registration(ProtectionMessage::OK);
            }
            UciNewGame => {
                self.set_from_position(FEN_STARTPOS.clone(), &[]);
            }
            PositionFrom { fen, moves } => {
                self.set_from_position(fen, &moves.into_iter().map(|mv| mv.san()).collect::<Vec<_>>()[..]);
            }
            GoCommand { go } => {
                self.start_new_search(go);
            }
            Stop => {
                todo!()
            }
            PonderHit => {
                todo!()
            }
            Quit => {
                todo!()
            }
        }
    }
}

struct SearchThreadHolder<T: UciTx + Send + Sync + 'static> {
    handle: Option<JoinHandle<()>>,
    search: Arc<Mutex<Search<T>>>,
    next_search_is_new: bool,
}

impl<T: UciTx + Send + Sync + 'static> SearchThreadHolder<T> {
    pub fn new(tx: Arc<T>) -> Self {
        Self { handle: None, search: Arc::new(Mutex::new(Search::new(tx))), next_search_is_new: true }
    }

    /// Start a search with the given parameters, returns whether or not starting the search was successful
    pub fn start(&mut self, fen: Fen, go: Go) -> bool {
        if self.is_running() {
            return false;
        }

        let search_arc = self.search.clone();
        let reset = self.next_search_is_new;
        let handle = thread::spawn(move || {
            search_arc.lock().unwrap().start(fen, go, reset);
        });

        self.next_search_is_new = false;
        self.handle.replace(handle);

        true
    }

    pub fn reset(&mut self) {
        self.next_search_is_new = true
    }

    pub fn wait_for(self) {
        if let Some(handle) = self.handle {
            handle.join().unwrap();
        }
    }

    pub fn is_running(&self) -> bool {
        match &self.handle {
            Some(handle) => !handle.is_finished(),
            None => false,
        }
    }

    fn search(&self) {}
}

struct Search<T: UciTx + Send + Sync + 'static> {
    tx: Arc<T>,
    buffers: Vec<Vec<Move>>,
}

impl<T: UciTx + Send + Sync + 'static> Search<T> {
    pub fn new(tx: Arc<T>) -> Self {
        Self { tx, buffers: Vec::new() }
    }

    fn buffer_for_depth(&mut self, depth: usize) -> &mut Vec<Move> {
        while self.buffers.len() <= depth {
            self.buffers.push(Vec::with_capacity(200));
        }

        let mut result = &mut self.buffers[depth];
        result.clear();
        result
    }

    pub fn start(&mut self, fen: Fen, go: Go, reset: bool) {
        let mut bitboard = Bitboard::new(&fen);

        let buffer = self.buffer_for_depth(0);
        bitboard.generate_pseudo_legal_moves_with_buffer(buffer);

        buffer.shuffle(&mut thread_rng());

        for mv in buffer {
            bitboard.make(*mv);
            if bitboard.is_valid() {
                let best_move = Some(move_into_uci_move(*mv));
                self.tx.best_move(best_move);
                return;
            }
            bitboard.unmake(*mv);
        }

        self.tx.best_move(None);
    }
}

fn move_into_uci_move(mv: Move) -> UciMove {
    match mv.structs() {
        (source, target, None) => UciMove::new(source, target),
        (source, target, Some(piece)) => UciMove::new_with_promotion(source, target, piece),
    }
}
