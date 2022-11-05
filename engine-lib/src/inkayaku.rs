use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

use rand::prelude::SliceRandom;
use rand::thread_rng;

use marvk_chess_board::board::{Bitboard, Move};
use marvk_chess_board::board::constants::ColorBits;
use marvk_chess_core::fen::{Fen, FEN_STARTPOS};
use marvk_chess_uci::uci::{Engine, Go, ProtectionMessage, UciCommand, UciMove, UciTx};
use SearchMessage::{UciGo, UciPositionFrom, UciUciNewGame};
use UciCommand::*;
use UciCommand::Go as GoCommand;

use crate::inkayaku::SearchMessage::{UciDebug, UciPonderHit, UciQuit, UciStop};
use crate::move_into_uci_move;

pub enum SearchMessage {
    UciUciNewGame,
    UciDebug(bool),
    UciPositionFrom(Fen, Vec<UciMove>),
    UciGo(Go),
    UciStop,
    UciPonderHit,
    UciQuit,
}

pub struct Inkayaku<T: UciTx + Send + Sync + 'static> {
    uci_tx: Arc<T>,
    debug: bool,
    search_tx: Sender<SearchMessage>,
    search_handle: Option<JoinHandle<()>>,
}

impl<T: UciTx + Send + Sync + 'static> Inkayaku<T> {
    pub fn new(uci_tx: T) -> Self {
        let uci_tx = Arc::new(uci_tx);
        let (search_tx, search_rx) = channel();
        let search_handle = Self::start_search_thread(search_rx, uci_tx.clone());

        Self { uci_tx, debug: false, search_tx, search_handle: Some(search_handle) }
    }

    fn start_search_thread(search_rx: Receiver<SearchMessage>, uci_tx: Arc<T>) -> JoinHandle<()> {
        thread::spawn(move || {
            Search::new(uci_tx, search_rx).idle();
        })
    }
}

impl<T: UciTx + Send + Sync + 'static> Engine for Inkayaku<T> {
    fn accept(&mut self, command: UciCommand) {
        match command {
            Uci => {
                self.uci_tx.id_name("Inkayaku");
                self.uci_tx.id_author("Marvin Kuhnke, check https://github.com/marvk/rust-chess");
                self.uci_tx.uci_ok();
            }
            SetDebug { debug } => {
                self.debug = debug;
                self.search_tx.send(UciDebug(debug));
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
                self.search_tx.send(UciUciNewGame);
            }
            PositionFrom { fen, moves } => {
                self.search_tx.send(UciPositionFrom(fen, moves));
            }
            GoCommand { go } => {
                self.search_tx.send(UciGo(go));
            }
            Stop => {
                self.search_tx.send(UciStop);
            }
            PonderHit => {
                self.search_tx.send(UciPonderHit);
            }
            Quit => {
                self.search_tx.send(UciQuit);
                self.search_handle.take().unwrap().join();
            }
        }
    }
}

struct Metrics {
    nodes: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            nodes: 0,
        }
    }
}

struct SearchState {
    bitboard: Bitboard,
    started_at: SystemTime,
    is_running: bool,
    buffers: RefCell<Vec<RefCell<Vec<Move>>>>,
    metrics: Metrics,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            bitboard: Bitboard::new(&FEN_STARTPOS.clone()),
            started_at: SystemTime::UNIX_EPOCH,
            is_running: false,
            buffers: RefCell::new(Vec::new()),
            metrics: Metrics::default(),
        }
    }
}

struct SearchFlags {
    reset_for_next_search: bool,
    stop_as_soon_as_possible: bool,
    quit_as_soon_as_possible: bool,
    ponder_hit: bool,
}

impl Default for SearchFlags {
    fn default() -> Self {
        Self {
            reset_for_next_search: false,
            stop_as_soon_as_possible: false,
            quit_as_soon_as_possible: false,
            ponder_hit: false,
        }
    }
}

struct SearchParams {
    go: Go,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            go: Go::EMPTY,
        }
    }
}

struct Search<T: UciTx + Send + Sync + 'static> {
    uci_tx: Arc<T>,
    search_rx: Receiver<SearchMessage>,
    debug: bool,
    state: SearchState,
    flags: SearchFlags,
    params: SearchParams,
}

impl<T: UciTx + Send + Sync + 'static> Search<T> {
    pub fn new(uci_tx: Arc<T>, rx: Receiver<SearchMessage>) -> Self {
        Self { uci_tx, search_rx: rx, debug: false, state: SearchState::default(), flags: SearchFlags::default(), params: SearchParams::default() }
    }

    fn idle(&mut self) {
        while !self.flags.quit_as_soon_as_possible {
            if let Ok(message) = self.search_rx.recv() {
                match message {
                    UciUciNewGame => {
                        self.flags.reset_for_next_search = true;
                    }
                    UciDebug(debug) => {
                        self.debug = debug;
                    }
                    UciPositionFrom(fen, moves) => {
                        self.state.bitboard = Bitboard::new(&fen);
                        self.state.bitboard.make_all_san(moves.iter().map(|mv| mv.san()).collect::<Vec<_>>().as_slice());
                    }
                    UciGo(go) => {
                        self.params.go = go;
                        self.go();
                    }
                    UciStop => {
                        // ignore during idle
                    }
                    UciPonderHit => {
                        // ignore during idle
                    }
                    UciQuit => {
                        self.flags.quit_as_soon_as_possible = true;
                    }
                }
            }
        }
    }

    fn check_channel(&mut self) {
        loop {
            if let Ok(message) = self.search_rx.try_recv() {
                match message {
                    UciUciNewGame => {
                        self.flags.reset_for_next_search = true;
                    }
                    UciDebug(debug) => {
                        self.debug = debug;
                    }
                    UciPositionFrom(_, _) => {
                        // Ignore positions send during go
                    }
                    UciGo(_) => {
                        // Ignore go send during go
                    }
                    UciStop => {
                        self.flags.stop_as_soon_as_possible = true;
                    }
                    UciPonderHit => {
                        self.flags.ponder_hit = true;
                    }
                    UciQuit => {
                        self.flags.stop_as_soon_as_possible = true;
                        self.flags.quit_as_soon_as_possible = true;
                    }
                }
            } else {
                return;
            }
        }
    }

    fn create_buffer(&self) -> Vec<Move> {
        Vec::with_capacity(200)
    }

    #[inline(always)]
    fn board(&mut self) -> &mut Bitboard {
        &mut self.state.bitboard
    }

    pub fn go(&mut self) {
        self.state.is_running = true;
        self.state.started_at = SystemTime::now();

        let best_move = self.best_move();
        self.uci_tx.best_move(best_move);

        self.state.is_running = false;
    }

    fn best_move(&mut self) -> Option<UciMove> {
        let mut buffer = self.create_buffer();
        self.board().generate_pseudo_legal_moves_with_buffer(&mut buffer);

        buffer.shuffle(&mut thread_rng());

        for mv in buffer {
            self.board().make(mv);
            if self.board().is_valid() {
                let best_move = Some(move_into_uci_move(mv));
                self.board().unmake(mv);
                return best_move;
            }
            self.board().unmake(mv);
        }

        None
    }

    fn negamax(depth: usize, alpha_original: i32, beta_original: i32, color: ColorBits) {}
}
