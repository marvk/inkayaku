use std::cmp::{max, min};
use std::ops::{Div, Mul};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime};

use marvk_chess_board::board::{Bitboard, Move};
use marvk_chess_board::board::constants::{ColorBits, WHITE, ZobristHash};
use marvk_chess_core::fen::Fen;
use marvk_chess_uci::uci::{Go, Info, Score, UciMove, UciTx};
use SearchMessage::{UciDebug, UciGo, UciPonderHit, UciPositionFrom, UciQuit, UciStop, UciUciNewGame};

use crate::inkayaku::heuristic::Heuristic;
use crate::inkayaku::metrics::{Metrics, MetricsService};
use crate::inkayaku::move_order::MoveOrder;
use crate::inkayaku::table::killer::KillerTable;
use crate::inkayaku::table::transposition::{HashMapTranspositionTable, TranspositionTable, TtEntry};
use crate::inkayaku::table::transposition::NodeType::{Exact, Lowerbound, Upperbound};
use crate::inkayaku::zobrist_history::ZobristHistory;
use crate::move_into_uci_move;

pub struct Search<T: UciTx, H: Heuristic, M: MoveOrder> {
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

    pub fn idle(&mut self) {
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

    /// Reset the search for the next go
    fn reset_for_go(&mut self) {
        if self.flags.reset_for_next_search {
            self.state.metrics = MetricsService::default();
            self.state.transposition_table.clear();
            self.state.killer_table.clear();
            self.flags.reset_for_next_search = false;
        } else {
            self.state.metrics.last = Metrics::default();
        }

        self.flags = SearchFlags::default();
    }

    // Start the search
    pub fn go(&mut self) {
        self.reset_for_go();

        self.state.is_running = true;
        self.state.started_at = SystemTime::now();

        let (best_move, ponder_move) = self.best_move();
        self.uci_tx.best_move(best_move, ponder_move);

        self.state.is_running = false;
    }

    // Time remaining of the engine
    fn get_self_time_remaining(&self) -> Option<Duration> {
        if self.state.bitboard.turn == WHITE { self.params.go.white_time } else { self.params.go.black_time }
    }

    /// Increment of the engine
    fn get_self_increment(&self) -> Option<Duration> {
        if self.state.bitboard.turn == WHITE { self.params.go.white_increment } else { self.params.go.black_increment }
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
        let increment = self.get_self_increment();
        let time_remaining = self.get_self_time_remaining();

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
        self.state.transposition_table.clear();
        self.state.killer_table.age(2);

        self.state.started_at = SystemTime::now();

        let mut best_move = None;

        if self.options.try_previous_pv {
            self.try_set_pv_from_continuation();
        }

        let max_depth = self.params.go.depth.map(|d| d as usize).unwrap_or(999999);

        if self.params.go.move_time.is_none() {
            self.params.go.move_time = self.calculate_max_thinking_time().map(|d| d.mul(2));
        }

        let max_thinking_time = self.params.go.move_time.unwrap_or(Duration::MAX);

        let mut uci_pv = None;
        let mut score = None;

        for depth in 1..=max_depth {
            let current_best_move = self.search_negamax(
                &mut self.create_buffer(),
                0,
                depth,
                self.heuristic.loss_score(),
                self.heuristic.win_score(),
                self.state.principal_variation.is_some(),
                self.state.bitboard.calculate_zobrist_hash(),
            );

            let elapsed = self.state.elapsed();

            let too_little_time = elapsed > max_thinking_time.div(3);
            let aborted = self.flags.stop_as_soon_as_possible || current_best_move.mv.is_none();
            let stop = aborted || too_little_time;

            if !stop {
                let bb_pv = current_best_move.calculate_principal_variation();
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
                depth: Some((if aborted { depth - 1 } else { depth }) as u32),
                string: self.generate_debug_string_if_enabled(),
                ..self.generate_info()
            });

            if stop {
                break;
            }
        }

        self.state.metrics.increment_duration(&self.state.elapsed());

        (best_move.and_then(|vm| vm.mv).map(move_into_uci_move), self.state.ponder_move().map(move_into_uci_move))
    }

    fn evaluate(&self, color: ColorBits, legal_moves_remaining: bool) -> i32 {
        calculate_heuristic_factor(color) * self.heuristic.evaluate(&self.state.bitboard, legal_moves_remaining)
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

    #[allow(clippy::too_many_arguments)]
    fn search_negamax(&mut self, buffer: &mut Vec<Move>, ply_depth_from_root: usize, max_ply: usize, alpha_original: i32, beta_original: i32, is_pv: bool, zobrist: ZobristHash) -> ValuedMove {
        let color = self.state.bitboard.turn;

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

        let ply_clock = self.state.bitboard.ply_clock();
        let halfmove_clock = self.state.bitboard.halfmove_clock;
        self.state.zobrist_history.set(ply_clock, zobrist);

        if self.state.zobrist_history.count_repetitions(ply_clock, halfmove_clock as u16) >= 3 {
            let contempt_factor_factor = if ply_depth_from_root % 2 == 0 { 1 } else { -1 };

            // todo if depth == ply, null move
            return ValuedMove::leaf(self.heuristic.draw_score() + contempt_factor_factor * self.options.contempt_factor);
        }

        let maybe_tt_entry = self.state.transposition_table.get(zobrist);

        let mut alpha = alpha_original;
        let mut beta = beta_original;

        let mut tt_move = None;

        let remaining_draft = max_ply - ply_depth_from_root;

        if let Some(tt_entry) = maybe_tt_entry {
            if tt_entry.depth >= remaining_draft {
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
            tt_move = tt_entry.mv.mv;
        };

        buffer.clear();
        self.state.bitboard.generate_pseudo_legal_moves_with_buffer(buffer);

        let is_root = ply_depth_from_root == 0;
        if is_root {
            self.filter_search_moves(buffer);

            if buffer.is_empty() {
                return ValuedMove::leaf(0);
            }
        }

        let is_max_ply = ply_depth_from_root == max_ply;
        if is_max_ply {
            let legal_moves_remaining = self.state.bitboard.is_any_move_legal(buffer);

            if legal_moves_remaining && Bitboard::is_any_move_non_quiescent(buffer) {
                self.state.metrics.increment_started_quiescence_search();
                return self.search_quiescence(0, buffer, alpha, beta);
            }

            let value = self.evaluate(color, legal_moves_remaining);
            return ValuedMove::leaf(value);
        }

        let pv_move = if is_pv { self.state.principal_variation.as_ref().unwrap().get(ply_depth_from_root).copied() } else { None };
        let killer_move = self.state.killer_table.get(remaining_draft);
        self.move_order.sort(buffer, pv_move, tt_move, killer_move);

        let mut best_value = self.heuristic.loss_score();
        let mut best_child: Option<ValuedMove> = None;
        let mut best_move: Option<Move> = None;
        let mut legal_moves_encountered = false;

        let mut next_buffer = self.create_buffer();

        for mv in buffer {
            self.state.bitboard.make(*mv);
            if !self.state.bitboard.is_valid() {
                self.state.bitboard.unmake(*mv);
                continue;
            }

            let zobrist_xor = Bitboard::zobrist_xor(*mv);

            legal_moves_encountered = true;

            let child = self.search_negamax(&mut next_buffer, ply_depth_from_root + 1, max_ply, -beta, -alpha, pv_move.map(|pv_mv| pv_mv.bits == mv.bits).unwrap_or(false), zobrist ^ zobrist_xor);

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

            self.state.bitboard.unmake(*mv);

            if alpha >= beta {
                self.state.killer_table.put(remaining_draft, *mv);
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

            self.state.transposition_table.put(zobrist, TtEntry::new(result.clone(), zobrist, remaining_draft, best_value, node_type));
        }

        // TODO transposition table

        result
    }

    fn search_quiescence(&mut self, depth: u32, buffer: &mut Vec<Move>, alpha_original: i32, beta_original: i32) -> ValuedMove {
        let color = self.state.bitboard.turn;

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
        self.state.bitboard.generate_pseudo_legal_non_quiescent_moves_with_buffer(buffer);
        self.move_order.sort(buffer, None, None, None);

        for mv in buffer {
            self.state.bitboard.make(*mv);

            if !self.state.bitboard.is_valid() {
                self.state.bitboard.unmake(*mv);
                continue;
            }

            self.state.metrics.increment_quiescence_nodes();

            let child = self.search_quiescence(depth + 1, &mut next_buffer, -beta_original, -alpha);
            let value = -child.value;

            self.state.bitboard.unmake(*mv);

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

/// Non-search related functionality
impl<T: UciTx, H: Heuristic, M: MoveOrder> Search<T, H, M> {
    fn generate_info(&self) -> Info {
        Info {
            nodes: Some(self.state.metrics.last.total_nodes()),
            hash_full: Some((self.state.transposition_table.load_factor() * 1000.0) as u32),
            nps: Some(self.state.metrics.last.nps_with_duration(&self.state.elapsed())),
            ..Info::EMPTY
        }
    }

    fn generate_debug_string_if_enabled(&self) -> Option<String> {
        if self.options.debug { Some(self.generate_debug_string()) } else { None }
    }

    fn generate_debug_string(&self) -> String {
        format!("tphitrate {} nrate {} qrate {} avgqdepth {} qstartedrate {} qtphitrate {}",
                self.state.metrics.last.table_hit_rate(),
                self.state.metrics.last.negamax_node_rate(),
                self.state.metrics.last.quiescence_node_rate(),
                self.state.metrics.last.average_quiescence_termination_ply(),
                self.state.metrics.last.quiescence_started_rate(),
                self.state.metrics.last.quiescence_table_hit_rate(),
        )
    }
}

#[inline(always)]
fn calculate_heuristic_factor(color: ColorBits) -> i32 {
    1 + (color as i32) * -2
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

    fn calculate_principal_variation(&self) -> Vec<&Move> {
        let mut result = Vec::new();

        let option = Some(self);
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
}

#[derive(Debug)]
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
pub struct EngineOptions {
    pub debug: bool,
    pub try_previous_pv: bool,
    pub contempt_factor: i32,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            debug: false,
            try_previous_pv: true,
            contempt_factor: 50,
        }
    }
}

/// State during search
struct SearchState {
    bitboard: Bitboard,
    transposition_table: HashMapTranspositionTable,
    killer_table: KillerTable,
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
            transposition_table: HashMapTranspositionTable::new(10_000_000),
            killer_table: KillerTable::default(),
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

#[cfg(test)]
mod test {
    use marvk_chess_board::board::constants::{BLACK, WHITE};

    use crate::inkayaku::search::calculate_heuristic_factor;

    #[test]
    fn test_heuristic_factor() {
        assert_eq!(calculate_heuristic_factor(BLACK), -1);
        assert_eq!(calculate_heuristic_factor(WHITE), 1);
    }
}
