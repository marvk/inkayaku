extern crate core;

use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Eq, Copy, Clone)]
struct PerftResult {
    nodes: u64,
    captures: u64,
    en_passant: u64,
    castles: u64,
    promotions: u64,
    checks: u64,
    discovery_checks: u64,
    double_checks: u64,
    checkmates: u64,
}

impl Debug for PerftResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PerftResult {{ nodes: {} }}",
            self.nodes
        )
    }
}

impl PerftResult {
    pub const EMPTY: Self = Self::new();

    pub const fn new() -> Self {
        Self { nodes: 0, captures: 0, en_passant: 0, castles: 0, promotions: 0, checks: 0, discovery_checks: 0, double_checks: 0, checkmates: 0 }
    }
}

const fn expect(nodes: u64) -> PerftResult {
    PerftResult {
        nodes,
        ..PerftResult::EMPTY
    }
}

pub mod perft {
    use std::time::{Duration, SystemTime};
    use std::usize;

    use marvk_chess_board::board::{Bitboard, Move};

    use crate::{expect, PerftResult};

    pub fn run_all() {
        perft1();

        println!("Warmup done");

        loop {
            time();
        }
    }

    fn time() {
        let start = SystemTime::now();

        perft1();
        perft2();
        perft3();
        perft4();
        perft5();
        perft6();
        perft7();

        println!("Full run: {:?}", start.elapsed());
    }

    pub fn perft1() {
        run_perft(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &[
                expect(20),
                expect(400),
                expect(8_902),
                expect(197_281),
                expect(4_865_609),
                expect(119_060_324),
            ],
        );
    }

    pub fn perft2() {
        run_perft(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
            &[
                expect(48),
                expect(2_039),
                expect(97_862),
                expect(4_085_603),
                expect(193_690_690),
            ],
        );
    }

    pub fn perft3() {
        run_perft(
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -",
            &[
                expect(14),
                expect(191),
                expect(2_812),
                expect(43_238),
                expect(674_624),
                expect(11_030_083),
                expect(178_633_661),
            ],
        );
    }

    pub fn perft4() {
        run_perft(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            &[
                expect(6),
                expect(264),
                expect(9_467),
                expect(422_333),
                expect(15_833_292),
                expect(706_045_033),
            ],
        );
    }

    pub fn perft5() {
        run_perft(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
            &[
                expect(6),
                expect(264),
                expect(9_467),
                expect(422_333),
                expect(15_833_292),
                expect(706_045_033),
            ],
        );
    }

    pub fn perft6() {
        run_perft(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            &[
                expect(44),
                expect(1_486),
                expect(62_379),
                expect(2_103_487),
                expect(89_941_194),
            ],
        );
    }

    pub fn perft7() {
        run_perft(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            &[
                expect(46),
                expect(2_079),
                expect(89_890),
                expect(3_894_594),
                expect(164_075_551),
            ],
        );
    }

    fn run_perft(fen_string: &str, expect: &[PerftResult]) {
        let start = SystemTime::now();

        let mut board = Bitboard::from_fen_string_unchecked(fen_string);

        let n = expect.len();
        let actual =
            (1..=n)
                .map(|index| {
                    let mut result = PerftResult::new();
                    run_perft_recursive(&mut board, &mut result, &mut Vec::new(), index);
                    result
                })
                .collect::<Vec<_>>();

        let nodes: u64 = expect.iter().map(|e| e.nodes).sum();

        assert_eq!(actual, expect, "Failed for {}", fen_string);
        let nps = nodes as f64 / start.elapsed().unwrap_or(Duration::ZERO).as_micros() as f64;
        println!("{:?} - {:.1} MM NPS", start.elapsed(), nps);
    }

    fn run_perft_recursive(board: &mut Bitboard, result: &mut PerftResult, buffer: &mut Vec<Move>, current_depth: usize) {
        if current_depth == 0 {
            result.nodes += 1;
            return;
        }

        board.generate_pseudo_legal_moves_with_buffer(buffer);

        let mut next_buffer = Vec::new();
        for mv in buffer {
            board.make(*mv);

            if board.is_valid() {
                run_perft_recursive(board, result, &mut next_buffer, current_depth - 1);
                next_buffer.clear();
            }

            board.unmake(*mv);
        }
    }
}

fn main() {
    perft::run_all();
}
