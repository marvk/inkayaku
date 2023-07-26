use std::{thread, usize};
use std::cmp::{max, min};
use std::ops::{Div, Mul};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use marvk_chess_board::board::{Bitboard, Move};
use marvk_chess_board::board::constants::{ColorBits, WHITE, ZobristHash};
use marvk_chess_core::fen::Fen;
use marvk_chess_uci::uci::{Engine, Go, Info, ProtectionMessage, Score, UciCommand, UciMove, UciTx};
use SearchMessage::{UciGo, UciPositionFrom, UciUciNewGame};
use UciCommand::*;
use UciCommand::Go as GoCommand;

use crate::inkayaku::heuristic::{Heuristic, SimpleHeuristic};
use crate::inkayaku::move_order::{MoveOrder, MvvLvaMoveOrder};
use crate::inkayaku::SearchMessage::{UciDebug, UciPonderHit, UciQuit, UciStop};
use crate::inkayaku::transposition_table::{TranspositionTable, TtEntry};
use crate::inkayaku::transposition_table::NodeType::{Exact, Lowerbound, Upperbound};
use crate::inkayaku::zobrist_history::ZobristHistory;
use crate::move_into_uci_move;

mod heuristic;
mod transposition_table;
mod move_order;
mod zobrist_history;

pub struct Inkayaku<T: UciTx + Send + Sync + 'static> {
    uci_tx: Arc<T>,
    debug: bool,
    search_tx: Sender<SearchMessage>,
    search_handle: Option<JoinHandle<()>>,
}

impl<T: UciTx + Send + Sync + 'static> Inkayaku<T> {
    pub fn new(uci_tx: Arc<T>, debug: bool) -> Self {
        let (search_tx, search_rx) = channel();
        let search_handle = Self::start_search_thread(search_rx, uci_tx.clone(), debug);

        Self { uci_tx, debug, search_tx, search_handle: Some(search_handle) }
    }

    fn start_search_thread(search_rx: Receiver<SearchMessage>, uci_tx: Arc<T>, debug: bool) -> JoinHandle<()> {
        thread::spawn(move || {
            Search::new(uci_tx, search_rx, SimpleHeuristic::default(), MvvLvaMoveOrder::default(), EngineOptions { debug, ..EngineOptions::default() }).idle();
        })
    }
}

impl<T: UciTx + Send + Sync + 'static> Engine for Inkayaku<T> {
    #[allow(unused_variables)]
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

struct Search<T: UciTx, H: Heuristic, M: MoveOrder> {
    uci_tx: Arc<T>,
    search_rx: Receiver<SearchMessage>,
    heuristic: H,
    move_order: M,

    state: SearchState,
    options: EngineOptions,
    flags: SearchFlags,
    params: SearchParams,
}

impl<T: UciTx, H: Heuristic, M: MoveOrder> Search<T, H, M> {
    pub fn new(uci_tx: Arc<T>, rx: Receiver<SearchMessage>, heuristic: H, move_order: M, options: EngineOptions) -> Self {
        Self { uci_tx, search_rx: rx, state: SearchState::default(), options, flags: SearchFlags::default(), params: SearchParams::default(), heuristic, move_order }
    }

    fn idle(&mut self) {
        while !self.flags.quit_as_soon_as_possible {
            if let Ok(message) = self.search_rx.recv() {
                match message {
                    UciUciNewGame => {
                        self.flags.reset_for_next_search = true;
                    }
                    UciDebug(debug) => {
                        self.options.debug = debug;
                    }
                    UciPositionFrom(fen, moves) => {
                        self.set_position_from(fen, moves);
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

    fn set_position_from(&mut self, fen: Fen, moves: Vec<UciMove>) {
        let mut board = Bitboard::new(&fen);
        let mut zobrist_history = ZobristHistory::default();
        zobrist_history.set(board.ply_clock(), board.calculate_zobrist_hash());

        let mut bb_moves = Vec::new();

        for uci in moves {
            match board.find_uci(&uci.to_string()) {
                Ok(mv) => {
                    board.make(mv);
                    zobrist_history.set(board.ply_clock(), board.calculate_zobrist_hash());
                    bb_moves.push(mv);
                }
                Err(error) => {
                    eprintln!("{:?}", error);
                    return;
                }
            };
        }

        self.state.bitboard = board;
        self.state.zobrist_history = zobrist_history;
        self.params.fen = fen;
        self.params.moves = bb_moves;
    }

    fn check_messages(&mut self) {
        loop {
            match self.search_rx.try_recv() {
                Ok(message) => match message {
                    UciUciNewGame => {
                        self.flags.reset_for_next_search = true;
                    }
                    UciDebug(debug) => {
                        self.options.debug = debug;
                    }
                    UciPositionFrom(..) => {
                        // Ignore positions send during go
                    }
                    UciGo(..) => {
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

        let (best_move, ponder_move) = self.best_move();
        self.uci_tx.best_move(best_move, ponder_move);

        self.state.is_running = false;
    }

    fn self_time_remaining(&self) -> Option<Duration> {
        if self.state.bitboard.turn == WHITE {
            self.params.go.white_time
        } else {
            self.params.go.black_time
        }
    }

    fn self_increment(&self) -> Option<Duration> {
        if self.state.bitboard.turn == WHITE {
            self.params.go.white_increment
        } else {
            self.params.go.black_increment
        }
    }

    /// Check if the last played move was the ponder move. If it was, calculate the current pv.
    fn try_set_pv_from_continuation(&mut self) {
        let last_ponder_move = self.state.ponder_move();
        let last_move_played = self.params.moves.last();

        let message = match (last_ponder_move, last_move_played) {
            (Some(last_ponder_move), Some(last_move_played)) => {
                if last_ponder_move.bits == last_move_played.bits {
                    let new_pv = self.state.principal_variation.take().unwrap().drain(0..2).collect();
                    let result = format!("successful ponder continuation with {}: {:?}", last_ponder_move, new_pv);
                    self.state.principal_variation = Some(new_pv);
                    result
                } else {
                    format!("failed ponder continuation from ponder {}, {} was played", last_ponder_move, last_move_played)
                }
            }
            (Some(last_ponder_move), None) => format!("failed ponder continuation from ponder {}, couldn't find last played move", last_ponder_move),
            _ => "failed ponder continuation, no ponder move".to_string(),
        };

        self.uci_tx.debug(&message);
    }


    fn calculate_max_thinking_time(&self) -> Option<Duration> {
        let increment = self.self_increment();
        let time_remaining = self.self_time_remaining();


        if let Some(time_remaining) = time_remaining {
            if let Some(increment) = increment {
                let increment_factor = match time_remaining.as_secs() {
                    20.. => 1.0,
                    10.. => 0.75,
                    2.. => 0.5,
                    _ => 0.25,
                };

                Some(increment.mul_f64(increment_factor))
            } else {
                Some(time_remaining.div(60))
            }
        } else {
            None
        }
    }

    fn best_move(&mut self) -> (Option<UciMove>, Option<UciMove>) {
        self.state.started_at = SystemTime::now();

        let mut best_move = None;

        if self.options.try_previous_pv {
            self.try_set_pv_from_continuation();
        }

        let max_depth = self.params.go.depth.map(|d| d as usize).unwrap_or(999999);

        if self.params.go.move_time.is_none() {
            self.params.go.move_time = self.calculate_max_thinking_time().map(|d| d.mul(2));
        }

        let mut uci_pv = None;
        let mut score = None;

        for depth in 1..=max_depth {
            let current_best_move = self.negamax(
                &mut self.create_buffer(),
                depth,
                depth,
                self.heuristic.loss_score(),
                self.heuristic.win_score(),
                self.state.principal_variation.is_some(),
                self.state.bitboard.calculate_zobrist_hash(),
            );

            let elapsed = self.state.elapsed();
            let max_thinking_time = self.params.go.move_time.unwrap_or(Duration::MAX);
            let stop = self.flags.stop_as_soon_as_possible || elapsed > max_thinking_time.div(3) || current_best_move.mv.is_none();
            if !stop {
                let bb_pv = principal_variation(&current_best_move);
                self.state.principal_variation = Some(bb_pv.iter().map(|&&mv| mv).collect());
                uci_pv = Some(bb_pv.into_iter().map(|&mv| move_into_uci_move(mv)).collect::<Vec<_>>());
                score = Some(
                    self.heuristic
                        .find_mate_at_fullmove_clock(current_best_move.value, &self.state.bitboard)
                        .unwrap_or(Score::Centipawn { score: current_best_move.value })
                );

                best_move = Some(current_best_move);
            }

            self.uci_tx.info(&Info {
                principal_variation: uci_pv.clone(),
                time: Some(elapsed),
                score,
                depth: Some((if stop { depth - 1 } else { depth }) as u32),
                string: self.debug_string(),
                ..self.generate_info()
            });

            if stop {
                break;
            }
        }

        self.state.metrics.increment_duration(&self.state.elapsed());

        (best_move.and_then(|vm| vm.mv).map(move_into_uci_move), self.state.ponder_move().map(move_into_uci_move))
    }

    fn generate_info(&self) -> Info {
        Info {
            nodes: Some(self.state.metrics.last.total_nodes()),
            hash_full: Some((self.state.transposition_table.load_factor() * 1000.0) as u32),
            nps: Some(self.state.metrics.last.nps_with_duration(&self.state.elapsed())),
            ..Info::EMPTY
        }
    }

    fn debug_string(&self) -> Option<String> {
        if self.options.debug { Some(self.generate_debug()) } else { None }
    }

    fn generate_debug(&self) -> String {
        format!("tphitrate {} nrate {} qrate {} avgqdepth {} qstartedrate {} qtphitrate {}",
                self.state.metrics.last.table_hit_rate(),
                self.state.metrics.last.negamax_node_rate(),
                self.state.metrics.last.quiescence_node_rate(),
                self.state.metrics.last.average_quiescence_termination_ply(),
                self.state.metrics.last.quiescence_started_rate(),
                self.state.metrics.last.quiescence_table_hit_rate(),
        )
    }

    fn evaluate(&self, color: ColorBits, legal_moves_remaining: bool) -> i32 {
        heuristic_factor(color) * self.heuristic.evaluate(&self.state.bitboard, legal_moves_remaining)
    }

    #[allow(clippy::too_many_arguments)]
    fn negamax(&mut self, buffer: &mut Vec<Move>, depth: usize, ply: usize, alpha_original: i32, beta_original: i32, pv: bool, zobrist: ZobristHash) -> ValuedMove {
        let color = self.board().turn;

        let check_flags = self.should_check_flags();
        if check_flags {
            self.check_messages();
            self.uci_tx.info(&Info {
                time: Some(self.state.elapsed()),
                ..self.generate_info()
            });

            if let Some(move_time) = self.params.go.move_time {
                if self.state.elapsed() > move_time {
                    self.flags.stop_as_soon_as_possible = true;
                    return ValuedMove::leaf(0);
                }
            }
        }

        self.state.metrics.increment_negamax_nodes();

        let ply_clock = self.board().ply_clock();
        let halfmove_clock = self.board().halfmove_clock;
        self.state.zobrist_history.set(ply_clock, zobrist);
        if self.state.zobrist_history.count_repetitions(ply_clock, halfmove_clock) >= 2 {
            let contempt_factor_factor = if (ply - depth) % 2 == 0 { 1 } else { -1 };

            // todo if depth == ply, null move
            return ValuedMove::leaf(self.heuristic.draw_score() + contempt_factor_factor * self.options.contempt_factor);
        }

        let maybe_tt_entry = self.state.transposition_table.get(zobrist);

        let mut alpha = alpha_original;
        let mut beta = beta_original;

        if let Some(tt_entry) = maybe_tt_entry {
            if tt_entry.depth >= depth {
                self.state.metrics.increment_transposition_hits();
                match tt_entry.node_type {
                    Lowerbound => alpha = max(alpha, tt_entry.value),
                    Upperbound => beta = min(beta, tt_entry.value),
                    Exact => {
                        return tt_entry.mv.clone();
                    }
                }
                if alpha >= beta {
                    return tt_entry.mv.clone();
                }
            }
        };

        buffer.clear();
        self.board().generate_pseudo_legal_moves_with_buffer(buffer);

        if ply == depth {
            self.filter_search_moves(buffer);

            if buffer.is_empty() {
                return ValuedMove::leaf(0);
            }
        }

        if depth == 0 {
            let legal_moves_remaining = self.board().is_any_move_legal(buffer);

            if legal_moves_remaining && Bitboard::is_any_move_non_quiescent(buffer) {
                self.state.metrics.increment_started_quiescence_search();
                return self.quiescence_search(0, buffer, alpha, beta);
            }

            let value = self.evaluate(color, legal_moves_remaining);
            return ValuedMove::leaf(value);
        }

        let pv_move = if pv { self.state.principal_variation.as_ref().unwrap().get(ply - depth).copied() } else { None };
        self.move_order.sort(buffer, pv_move);

        let mut best_value = self.heuristic.loss_score();
        let mut best_child: Option<ValuedMove> = None;
        let mut best_move: Option<Move> = None;
        let mut legal_moves_encountered = false;

        let mut next_buffer = self.create_buffer();

        for mv in buffer {
            self.board().make(*mv);
            if !self.board().is_valid() {
                self.board().unmake(*mv);
                continue;
            }

            let zobrist_xor = Bitboard::zobrist_xor(*mv);

            legal_moves_encountered = true;

            let child = self.negamax(&mut next_buffer, depth - 1, ply, -beta, -alpha, pv_move.map(|pv_mv| pv_mv.bits == mv.bits).unwrap_or(false), zobrist ^ zobrist_xor);

            if self.flags.stop_as_soon_as_possible {
                return ValuedMove::new(0, None, None);
            }

            let child_value = -child.value;

            if child_value > best_value {
                best_value = child_value;
                best_move = Some(*mv);
                best_child = Some(child);
            }

            alpha = max(alpha, best_value);

            self.board().unmake(*mv);


            if alpha >= beta {
                break;
            }
        }

        if !legal_moves_encountered {
            let value = self.evaluate(color, false);
            return ValuedMove::leaf(value);
        }

        let result = ValuedMove::new(best_value, best_move, best_child);

        if !self.heuristic.is_checkmate(best_value) {
            let node_type = if best_value <= alpha_original {
                Upperbound
            } else if best_value >= beta {
                Lowerbound
            } else {
                Exact
            };

            self.state.transposition_table.put(zobrist, TtEntry::new(result.clone(), depth, best_value, node_type));
        }

        // TODO transposition table

        result
    }

    #[inline(always)]
    fn should_check_flags(&mut self) -> bool {
        self.state.metrics.last.negamax_nodes % 100000 == 0 && self.state.metrics.last.negamax_nodes > 0
    }

    fn filter_search_moves(&mut self, buffer: &mut Vec<Move>) {
        let search_moves = &self.params.go.search_moves;

        if !search_moves.is_empty() {
            buffer.retain(|&mv| {
                search_moves.contains(&move_into_uci_move(mv))
            });
        }
    }

    fn quiescence_search(&mut self, depth: u32, buffer: &mut Vec<Move>, alpha_original: i32, beta_original: i32) -> ValuedMove {
        let color = self.board().turn;

        // TODO take attack moves from buffer on first call

        let standing_pat = self.evaluate(color, true);

        if standing_pat >= beta_original {
            self.state.metrics.register_quiescence_termination(depth as usize);
            return ValuedMove::leaf(beta_original);
        }

        let mut alpha = max(alpha_original, standing_pat);

        let mut best_move = None;
        let mut best_child = None;

        let mut next_buffer = Vec::new();

        buffer.clear();
        self.board().generate_pseudo_legal_non_quiescent_moves_with_buffer(buffer);
        self.move_order.sort(buffer, None);

        for mv in buffer {
            self.board().make(*mv);

            if !self.board().is_valid() {
                self.board().unmake(*mv);
                continue;
            }

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

pub enum SearchMessage {
    UciUciNewGame,
    UciDebug(bool),
    UciPositionFrom(Fen, Vec<UciMove>),
    UciGo(Go),
    UciStop,
    UciPonderHit,
    UciQuit,
}

/// UCI options
struct EngineOptions {
    debug: bool,
    try_previous_pv: bool,
    contempt_factor: i32,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            debug: false,
            try_previous_pv: true,
            contempt_factor: -99999,
        }
    }
}

/// State during search
struct SearchState {
    bitboard: Bitboard,
    transposition_table: TranspositionTable,
    principal_variation: Option<Vec<Move>>,
    zobrist_history: ZobristHistory,
    started_at: SystemTime,
    is_running: bool,
    metrics: MetricsService,
}

impl SearchState {
    fn ponder_move(&self) -> Option<Move> {
        self.principal_variation.as_ref().and_then(|pv| pv.get(1)).cloned()
    }

    fn elapsed(&self) -> Duration {
        self.started_at.elapsed().unwrap()
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            bitboard: Bitboard::default(),
            transposition_table: TranspositionTable::new(10_000_000),
            principal_variation: None,
            zobrist_history: ZobristHistory::default(),
            started_at: SystemTime::UNIX_EPOCH,
            is_running: false,
            metrics: MetricsService::default(),
        }
    }
}

/// Control the search "from the outside"
#[derive(Default)]
struct SearchFlags {
    reset_for_next_search: bool,
    stop_as_soon_as_possible: bool,
    quit_as_soon_as_possible: bool,
    ponder_hit: bool,
}

/// Input for the search
#[derive(Default)]
struct SearchParams {
    go: Go,
    fen: Fen,
    moves: Vec<Move>,
}

#[derive(Default)]
struct Metrics {
    negamax_nodes: u64,
    quiescence_nodes: u64,
    duration: Duration,
    transposition_hits: u64,
    quiescence_transposition_hits: u64,
    quiescence_termination_ply_sum: u64,
    quiescence_termination_count: u64,
    started_quiescence_search_count: u64,
}

impl Metrics {
    fn total_nodes(&self) -> u64 {
        self.negamax_nodes + self.quiescence_nodes
    }

    fn nps(&self) -> u64 {
        self.nps_with_duration(&self.duration)
    }

    fn nps_with_duration(&self, duration: &Duration) -> u64 {
        ((self.total_nodes() as f64 / duration.as_nanos() as f64) * 1_000_000_000.0) as u64
    }

    fn table_hit_rate(&self) -> f64 {
        self.transposition_hits as f64 / ((self.transposition_hits + self.negamax_nodes) as f64)
    }

    fn quiescence_table_hit_rate(&self) -> f64 {
        self.quiescence_transposition_hits as f64 / ((self.quiescence_transposition_hits + self.quiescence_nodes) as f64)
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

    fn increment_quiescence_transposition_hits(&mut self) {
        self.last.quiescence_transposition_hits += 1;
        self.total.quiescence_transposition_hits += 1;
    }

    fn register_quiescence_termination(&mut self, ply: usize) {
        self.last.quiescence_termination_ply_sum += ply as u64;
        self.last.quiescence_termination_count += 1;
        self.total.quiescence_termination_ply_sum += ply as u64;
        self.total.quiescence_termination_count += 1;
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::sync::mpsc::channel;

    use marvk_chess_board::board::constants::{BLACK, WHITE};
    use marvk_chess_core::fen::{Fen, FEN_STARTPOS};
    use marvk_chess_uci::uci::{Engine, Go, Score, UciCommand, UciMove, UciTxCommand};
    use marvk_chess_uci::uci::command::CommandUciTx;

    use crate::inkayaku::{heuristic_factor, Inkayaku};

    #[test]
    fn test_threefold_1() {
        let fen = Fen::new("5rk1/5r2/p7/2pNp1q1/2P1P2p/1P3P1P/P4RP1/5RK1 w - - 0 28").unwrap();
        let moves = vec![
            "d5b6", "g5e3", "b6d5", "e3g5",
            "d5b6", "g5e3", "b6d5",
        ];
        let move_to_draw = "e3g5";
        _test_threefold(moves, fen, move_to_draw);
    }

    #[test]
    fn test_threefold_2() {
        let fen = FEN_STARTPOS.clone();
        let moves = vec!["e2e4", "b8c6", "g1f3", "g8f6", "e4e5", "f6d5", "d2d4", "d7d6", "c2c4", "d5b6", "b1c3", "c8g4", "c1f4", "e7e6", "d1d3", "g4f3", "g2f3", "d6e5", "d4e5", "d8d4", "d3d4", "c6d4", "e1c1", "c7c5", "f4g3", "e8e7", "f3f4", "d4f5", "f1d3", "f5g3", "h2g3", "f7f6", "c1d2", "f6e5", "f4e5", "b6d7", "f2f4", "d7b6", "d2e3", "e7f7", "g3g4", "f8e7", "a2a4", "a7a5", "d1e1", "a8d8", "g4g5", "d8a8", "h1h3", "h7h6", "h3f3", "h6g5", "f4g5", "f7e8", "f3g3", "h8h2", "e1e2", "h2e2", "d3e2", "e8d7", "g3g4", "a8h8", "e2f3", "h8h3", "b2b3", "d7c7", "c3b5", "c7d7", "g4g2", "d7c8", "e3f4", "h3h4", "g2g4", "h4h3", "g4g2", "h3h4", "g2g4", "h4h3"];
        let move_to_draw = "g4g2";
        _test_threefold(moves, fen, move_to_draw);
    }

    #[test]
    fn test_threefold_3() {
        let fen = Fen::new("5r1k/5r2/p7/2pNp1q1/2P1P2p/1P3P1P/P4RP1/5RK1 b - - 0 28").unwrap();
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
        let mut engine = Inkayaku::new(Arc::new(CommandUciTx::new(tx)), false);

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


    #[test]
    fn test_heuristic_factor() {
        assert_eq!(heuristic_factor(BLACK), -1);
        assert_eq!(heuristic_factor(WHITE), 1);
    }
}
