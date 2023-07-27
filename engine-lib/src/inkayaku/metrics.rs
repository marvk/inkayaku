use std::time::Duration;

#[derive(Default)]
pub struct Metrics {
    pub negamax_nodes: u64,
    pub quiescence_nodes: u64,
    pub duration: Duration,
    pub transposition_hits: u64,
    pub quiescence_transposition_hits: u64,
    pub quiescence_termination_ply_sum: u64,
    pub quiescence_termination_count: u64,
    pub started_quiescence_search_count: u64,
}

impl Metrics {
    pub fn total_nodes(&self) -> u64 {
        self.negamax_nodes + self.quiescence_nodes
    }

    pub fn nps(&self) -> u64 {
        self.nps_with_duration(&self.duration)
    }

    pub fn nps_with_duration(&self, duration: &Duration) -> u64 {
        ((self.total_nodes() as f64 / duration.as_nanos() as f64) * 1_000_000_000.0) as u64
    }

    pub fn table_hit_rate(&self) -> f64 {
        self.transposition_hits as f64 / ((self.transposition_hits + self.negamax_nodes) as f64)
    }

    pub fn quiescence_table_hit_rate(&self) -> f64 {
        self.quiescence_transposition_hits as f64 / ((self.quiescence_transposition_hits + self.quiescence_nodes) as f64)
    }

    pub fn average_quiescence_termination_ply(&self) -> f64 {
        self.quiescence_termination_ply_sum as f64 / self.quiescence_termination_count as f64
    }

    pub fn negamax_node_rate(&self) -> f64 {
        self.negamax_nodes as f64 / self.total_nodes() as f64
    }

    pub fn quiescence_node_rate(&self) -> f64 {
        self.quiescence_nodes as f64 / self.total_nodes() as f64
    }

    pub fn quiescence_started_rate(&self) -> f64 {
        self.started_quiescence_search_count as f64 / self.negamax_nodes as f64
    }
}

#[derive(Default)]
pub struct MetricsService {
    pub last: Metrics,
    pub total: Metrics,
}

impl MetricsService {
    pub fn increment_negamax_nodes(&mut self) {
        self.last.negamax_nodes += 1;
        self.total.negamax_nodes += 1;
    }

    pub fn increment_quiescence_nodes(&mut self) {
        self.last.quiescence_nodes += 1;
        self.total.quiescence_nodes += 1;
    }

    pub fn increment_duration(&mut self, duration: &Duration) {
        self.last.duration = Duration::from_nanos((self.last.duration.as_nanos() + duration.as_nanos()) as u64);
        self.total.duration = Duration::from_nanos((self.total.duration.as_nanos() + duration.as_nanos()) as u64);
    }

    pub fn increment_transposition_hits(&mut self) {
        self.last.transposition_hits += 1;
        self.total.transposition_hits += 1;
    }

    pub fn increment_started_quiescence_search(&mut self) {
        self.last.started_quiescence_search_count += 1;
        self.total.started_quiescence_search_count += 1;
    }

    pub fn increment_quiescence_transposition_hits(&mut self) {
        self.last.quiescence_transposition_hits += 1;
        self.total.quiescence_transposition_hits += 1;
    }

    pub fn register_quiescence_termination(&mut self, ply: usize) {
        self.last.quiescence_termination_ply_sum += ply as u64;
        self.last.quiescence_termination_count += 1;
        self.total.quiescence_termination_ply_sum += ply as u64;
        self.total.quiescence_termination_count += 1;
    }
}
