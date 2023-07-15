use std::cell::RefCell;
use std::cmp::{max, min};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use marvk_chess_board::board::{Bitboard, Move, PlayerState};
use marvk_chess_board::board::constants::{BLACK, ColorBits, WHITE};
use marvk_chess_board::move_to_san;
use marvk_chess_core::fen::Fen;
use marvk_chess_uci::uci::{CurrentLine, Engine, Go, Info, ProtectionMessage, UciCommand, UciMove, UciTx};
use SearchMessage::{UciGo, UciPositionFrom, UciUciNewGame};
use UciCommand::*;
use UciCommand::Go as GoCommand;

use crate::inkayaku::heuristic::{Heuristic, SimpleHeuristic};
use crate::inkayaku::move_order::{MoveOrder, MvvLvaMoveOrder};
use crate::inkayaku::SearchMessage::{UciDebug, UciPonderHit, UciQuit, UciStop};
use crate::inkayaku::transposition_table::{NodeType, TranspositionTable, TtEntry};
use crate::inkayaku::transposition_table::NodeType::{EXACT, LOWERBOUND, UPPERBOUND};
use crate::move_into_uci_move;

mod heuristic;
mod transposition_table;
mod move_order;

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
    pub fn new(uci_tx: Arc<T>) -> Self {
        let (search_tx, search_rx) = channel();
        let search_handle = Self::start_search_thread(search_rx, uci_tx.clone());

        Self { uci_tx, debug: false, search_tx, search_handle: Some(search_handle) }
    }

    fn start_search_thread(search_rx: Receiver<SearchMessage>, uci_tx: Arc<T>) -> JoinHandle<()> {
        thread::spawn(move || {
            Search::new(uci_tx, search_rx, SimpleHeuristic {}, MvvLvaMoveOrder {}).idle();
        })
    }
}

impl<T: UciTx + Send + Sync + 'static> Engine for Inkayaku<T> {
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

#[derive(Default)]
struct Metrics {
    negamax_nodes: u64,
    quiescence_nodes: u64,
    duration: Duration,
    transposition_hits: u64,
    quiescence_termination_ply_sum: u64,
    quiescence_termination_count: u64,
    started_quiescence_search_count: u64,
}

impl Metrics {
    fn total_nodes(&self) -> u64 {
        self.negamax_nodes + self.quiescence_nodes
    }

    fn nps(&self) -> u64 {
        ((self.negamax_nodes as f64 / self.duration.as_nanos() as f64) * 1_000_000_000.0) as u64
    }

    fn table_hit_rate(&self) -> f64 {
        self.transposition_hits as f64 / ((self.transposition_hits + self.negamax_nodes) as f64)
    }

    fn average_quiescence_termination_ply(&self) -> f64 {
        self.quiescence_termination_ply_sum as f64 / self.quiescence_termination_count as f64
    }

    fn negamax_node_rate(&self) -> f64 {
        self.negamax_nodes as f64 / self.total_nodes() as f64
    }

    fn quiescence_node_rate(&self) -> f64 {
        self.quiescence_nodes as f64 / self.total_nodes() as f64
    }

    fn quiescence_started_rate(&self) -> f64 {
        self.started_quiescence_search_count as f64 / self.negamax_nodes as f64
    }
}

#[derive(Default)]
struct MetricsService {
    last: Metrics,
    total: Metrics,
}

impl MetricsService {
    fn increment_negamax_nodes(&mut self) {
        self.last.negamax_nodes += 1;
        self.total.negamax_nodes += 1;
    }

    fn increment_quiescence_nodes(&mut self) {
        self.last.quiescence_nodes += 1;
        self.total.quiescence_nodes += 1;
    }

    fn increment_duration(&mut self, duration: &Duration) {
        self.last.duration = Duration::from_nanos((self.last.duration.as_nanos() + duration.as_nanos()) as u64);
        self.total.duration = Duration::from_nanos((self.total.duration.as_nanos() + duration.as_nanos()) as u64);
    }

    fn increment_transposition_hits(&mut self) {
        self.last.transposition_hits += 1;
        self.total.transposition_hits += 1;
    }

    fn increment_started_quiescence_search(&mut self) {
        self.last.started_quiescence_search_count += 1;
        self.total.started_quiescence_search_count += 1;
    }

    fn register_quiescence_termination(&mut self, ply: usize) {
        self.last.quiescence_termination_ply_sum += ply as u64;
        self.last.quiescence_termination_count += 1;
        self.total.quiescence_termination_ply_sum += ply as u64;
        self.total.quiescence_termination_count += 1;
    }
}

struct SearchState {
    bitboard: Bitboard,
    transposition_table: TranspositionTable,
    started_at: SystemTime,
    is_running: bool,
    metrics: MetricsService,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            bitboard: Bitboard::default(),
            transposition_table: TranspositionTable::new(10_000_000),
            started_at: SystemTime::UNIX_EPOCH,
            is_running: false,
            metrics: MetricsService::default(),
        }
    }
}

#[derive(Default)]
struct SearchFlags {
    reset_for_next_search: bool,
    stop_as_soon_as_possible: bool,
    quit_as_soon_as_possible: bool,
    ponder_hit: bool,
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

struct Search<T: UciTx, H: Heuristic, M: MoveOrder> {
    uci_tx: Arc<T>,
    search_rx: Receiver<SearchMessage>,
    debug: bool,
    heuristic: H,
    move_order: M,
    state: SearchState,
    flags: SearchFlags,
    params: SearchParams,
}

impl<T: UciTx, H: Heuristic, M: MoveOrder> Search<T, H, M> {
    pub fn new(uci_tx: Arc<T>, rx: Receiver<SearchMessage>, heuristic: H, move_order: M) -> Self {
        Self { uci_tx, search_rx: rx, debug: false, state: SearchState::default(), flags: SearchFlags::default(), params: SearchParams::default(), heuristic, move_order }
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

    fn check_messages(&mut self) {
        loop {
            match self.search_rx.try_recv() {
                Ok(message) => match message {
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
                },
                Err(error) => {
                    self.uci_tx.debug(&format!("{}", error));
                    return;
                }
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

    fn reset_for_go(&mut self) {
        if self.flags.reset_for_next_search {
            self.state.metrics = MetricsService::default();
            self.state.transposition_table.clear();
            self.flags.reset_for_next_search = false;
        } else {
            self.state.metrics.last = Metrics::default();
        }

        self.flags = SearchFlags::default();
    }

    pub fn go(&mut self) {
        self.reset_for_go();

        self.state.is_running = true;
        self.state.started_at = SystemTime::now();

        let best_move = self.best_move();
        self.uci_tx.best_move(best_move);

        self.state.is_running = false;
    }

    fn best_move(&mut self) -> Option<UciMove> {
        self.state.started_at = SystemTime::now();

        let ply = self.params.go.depth.unwrap_or(4) as usize;

        let best_move = self.negamax(&mut self.create_buffer(), ply, ply, self.heuristic.loss_score(), self.heuristic.win_score());
        self.state.metrics.increment_duration(&self.state.started_at.elapsed().unwrap());

        let vec = principal_variation(&best_move).into_iter().map(|&mv| move_into_uci_move(mv)).collect::<Vec<_>>();

        self.uci_tx.info(&Info {
            current_line: Some(CurrentLine::new(1, vec)),
            ..self.generate_info()
        });

        best_move.mv.map(move_into_uci_move)
    }

    fn generate_info(&self) -> Info {
        Info {
            nodes: Some(self.state.metrics.last.total_nodes()),
            hash_full: Some((self.state.transposition_table.load_factor() * 1000.0) as u32),
            nps: Some(((self.state.metrics.last.total_nodes() as f64 / self.state.started_at.elapsed().unwrap().as_nanos() as f64) * 1_000_000_000.0) as u64),
            string: Some(format!("tphitrate {} nrate {} qrate {} avgqdepth {} qstartedrate {}",
                                 self.state.metrics.last.table_hit_rate(),
                                 self.state.metrics.last.negamax_node_rate(),
                                 self.state.metrics.last.quiescence_node_rate(),
                                 self.state.metrics.last.average_quiescence_termination_ply(),
                                 self.state.metrics.last.quiescence_started_rate(),
            )),
            ..Info::EMPTY
        }
    }

    fn negamax(&mut self, buffer: &mut Vec<Move>, depth: usize, ply: usize, alpha_original: i32, beta_original: i32) -> ValuedMove {
        if self.state.metrics.last.negamax_nodes % 1000000 == 0 {
            self.check_messages();
            self.uci_tx.info(&self.generate_info());
        }

        buffer.clear();
        self.state.metrics.increment_negamax_nodes();

        let zobrist = self.state.bitboard.zobrist_hash();

        let maybe_tt_entry = self.state.transposition_table.get(zobrist);

        let mut alpha = alpha_original;
        let mut beta = beta_original;
        println!("{}{}: initial alpha  {}", "    ".repeat(ply - depth), depth, alpha_original);
        println!("{}{}: initial beta   {}", "    ".repeat(ply - depth), depth, beta_original);

        if let Some(tt_entry) = maybe_tt_entry {
            if tt_entry.depth >= depth {
                self.state.metrics.increment_transposition_hits();
                match tt_entry.node_type {
                    NodeType::LOWERBOUND => alpha = max(alpha, tt_entry.value),
                    NodeType::UPPERBOUND => beta = min(beta, tt_entry.value),
                    NodeType::EXACT => {
                        println!("{}{}: TT Hit", "    ".repeat(ply - depth), depth);
                        return tt_entry.mv.clone();
                    }
                }
                if alpha >= beta {
                    println!("{}{}: TT Hit", "    ".repeat(ply - depth), depth);
                    return tt_entry.mv.clone();
                }
            } else {}
        };

        let color = self.state.bitboard.turn;

        self.board().generate_pseudo_legal_moves_with_buffer(buffer);

        buffer.sort_by_key(|mv| mv.san());

        println!("{}{}: {:?}", "    ".repeat(ply - depth), depth, buffer);

        if depth == 0 {
            let legal_moves_remaining = self.board().is_any_move_legal(buffer);

            if legal_moves_remaining && self.board().is_any_move_non_quiescent(buffer) {
                self.state.metrics.increment_started_quiescence_search();
                println!("{}{}: Q ENTRY", "    ".repeat(ply - depth), depth);
                println!("{}{}: initial alpha  {}", "    ".repeat(ply - depth), depth, alpha_original);
                println!("{}{}: initial beta   {}", "    ".repeat(ply - depth), depth, beta_original);
                return self.quiescence_search(0, buffer, alpha, beta);
            }

            let value = self::heuristic_factor(color) * self.heuristic.evaluate(&self.state.bitboard, legal_moves_remaining);
            println!("{}{}: OUT OF DEPTH VALUE {}", "    ".repeat(ply - depth), depth, value);
            return ValuedMove::leaf(value);
        }

        self.move_order.sort(buffer);
        println!("{}{}: {:?} (sorted)", "    ".repeat(ply - depth), depth, buffer);

        let mut best_value = self.heuristic.loss_score();
        let mut best_child: Option<ValuedMove> = None;
        let mut best_move: Option<Move> = None;
        let mut legal_moves_encountered = false;

        let mut next_buffer = self.create_buffer();

        for mv in buffer {
            self.board().make(*mv);
            println!("{}{}: {}", "    ".repeat(ply - depth), depth, mv.san());
            if !self.board().is_valid() {
                self.board().unmake(*mv);
                continue;
            }

            legal_moves_encountered = true;

            let child = self.negamax(&mut next_buffer, depth - 1, ply, -beta, -alpha);

            let child_value = -child.value;

            if child_value > best_value {
                best_value = child_value;
                best_move = Some(*mv);
                best_child = Some(child);
            }

            alpha = max(alpha, best_value);

            self.board().unmake(*mv);

            println!("{}{}: alpha {} / beta {}", "    ".repeat(ply - depth), depth, alpha, beta);

            if alpha >= beta {
                println!("{}{}: PRUNE", "    ".repeat(ply - depth), depth);
                break;
            }
        }

        if !legal_moves_encountered {
            let value = self::heuristic_factor(color) * self.heuristic.evaluate(&self.state.bitboard, false);
            println!("{}{}: OUT OF LEGAL MOVES VALUE {}", "    ".repeat(ply - depth), depth, value);
            return ValuedMove::leaf(value);
        }

        let result = ValuedMove::new(best_value, best_move, best_child);

        if !self.heuristic.is_checkmate(best_value) {
            let node_type = if best_value <= alpha_original {
                UPPERBOUND
            } else if best_value >= beta {
                LOWERBOUND
            } else {
                EXACT
            };

            self.state.transposition_table.put(zobrist, TtEntry::new(result.clone(), depth, best_value, node_type));
        }

        // TODO transposition table

        println!("{}{}: TERMINATION VALUE {}", "    ".repeat(ply - depth), depth, best_value);
        result
    }


    fn quiescence_search(&mut self, depth: u32, buffer: &mut Vec<Move>, alpha_original: i32, beta_original: i32) -> ValuedMove {
        println!("                    {}{}: {}", "    ".repeat(depth as usize), depth, self.state.bitboard.fen().fen);
        buffer.clear();

        // TODO take attack moves from buffer on first call
        self.board().generate_pseudo_legal_non_quiescent_moves_with_buffer(buffer);

        buffer.sort_by_key(|mv| mv.san());
        println!("                    {}{}: {:?}", "    ".repeat(depth as usize), depth, buffer);

        let standing_pat = self::heuristic_factor(self.state.bitboard.turn) * self.heuristic.evaluate(&self.state.bitboard, true);

        println!("                    {}{}: standing pat   {}", "    ".repeat(depth as usize), depth, standing_pat);
        println!("                    {}{}: initial alpha  {}", "    ".repeat(depth as usize), depth, alpha_original);
        println!("                    {}{}: initial beta   {}", "    ".repeat(depth as usize), depth, beta_original);

        if standing_pat >= beta_original {
            self.state.metrics.register_quiescence_termination(depth as usize);
            return ValuedMove::leaf(beta_original);
        }

        self.move_order.sort(buffer);
        println!("                    {}{}: {:?} (sorted)", "    ".repeat(depth as usize), depth, buffer);

        let mut alpha = max(alpha_original, standing_pat);

        let mut best_move = None;
        let mut best_child = None;

        let mut next_buffer = Vec::new();

        for mv in buffer {
            self.board().make(*mv);

            if !self.board().is_valid() {
                self.board().unmake(*mv);
                continue;
            }

            println!("                    {}{}: {}", "    ".repeat(depth as usize), depth, mv.san());

            self.state.metrics.increment_quiescence_nodes();

            let child = self.quiescence_search(depth + 1, &mut next_buffer, -beta_original, -alpha);
            let value = -child.value;

            self.board().unmake(*mv);

            if value >= beta_original {
                self.state.metrics.register_quiescence_termination(depth as usize);
                return ValuedMove::parent(beta_original, *mv, child);
            }

            if value > alpha {
                alpha = value;
                best_move = Some(*mv);
                best_child = Some(child);
            }
        }

        self.state.metrics.register_quiescence_termination(depth as usize);
        ValuedMove::new(alpha, best_move, best_child)
    }
}

#[inline(always)]
fn heuristic_factor(color: ColorBits) -> i32 {
    1 + (color as i32) * -2
}

fn principal_variation(valued_move: &ValuedMove) -> Vec<&Move> {
    let mut result = Vec::new();

    let option = Some(valued_move);
    let mut maybe_current = option;

    while maybe_current.is_some() {
        let current = maybe_current.unwrap();
        if let Some(mv) = &current.mv {
            result.push(mv);
        }


        maybe_current = (*current.pv_child).as_ref();
    }

    result
}

#[derive(Clone)]
pub struct ValuedMove {
    value: i32,
    mv: Option<Move>,
    pv_child: Box<Option<ValuedMove>>,
}

impl ValuedMove {
    pub fn new(value: i32, mv: Option<Move>, pv_child: Option<ValuedMove>) -> Self {
        Self { value, mv, pv_child: Box::new(pv_child) }
    }

    pub fn parent(value: i32, mv: Move, pv_child: ValuedMove) -> Self {
        Self::new(value, Some(mv), Some(pv_child))
    }

    pub fn leaf(value: i32) -> Self {
        Self::new(value, None, None)
    }
}


#[cfg(test)]
mod test {
    use marvk_chess_board::board::constants::{BLACK, WHITE};

    use crate::inkayaku::heuristic_factor;

    #[test]
    fn test_heuristic_factor() {
        assert_eq!(heuristic_factor(BLACK), -1);
        assert_eq!(heuristic_factor(WHITE), 1);
    }
}
