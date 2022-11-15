extern crate core;

use std::fmt::{Debug, Formatter};
use std::io::Read;
use std::process::{Command, Stdio};
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;

use marvk_chess_core::fen::Fen;

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

struct ReferenceEngine {
    path: &'static str,
}

impl ReferenceEngine {
    fn perft(&self, fen: &Fen, depth: usize) -> Vec<(String, u64)> {
        let mut child =
            Command::new(self.path)
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .expect("Failed to spawn child");

        let stdout = child.stdout.as_mut().unwrap();
        let mut stdin = child.stdin.take().unwrap();

        use std::io::Write;
        writeln!(
            &mut stdin,
            "position fen {}",
            fen.fen,
        ).unwrap();

        writeln!(
            &mut stdin,
            "go perft {}",
            depth,
        ).unwrap();

        sleep(Duration::from_secs(2));

        let mut buf = [0_u8; 65536];
        let len = stdout.read(&mut buf).unwrap();
        let result = from_utf8(&buf);
        let x = &(result.unwrap())[0..len];

        let result =
            x.lines()
                .skip(1)
                .take_while(|line| line.contains(':'))
                .map(|line| {
                    let mut split = line.split(':');
                    (split.next().unwrap().to_string(), split.next().unwrap().trim().parse().unwrap())
                }).collect::<Vec<_>>();

        child.kill().unwrap();

        result
    }

    pub const fn new(path: &'static str) -> Self {
        Self { path }
    }
}


#[cfg(test)]
mod perft_debug {
    use std::collections::HashSet;

    use marvk_chess_board::{move_to_san};
    use marvk_chess_board::board::{Bitboard, Move};
    use marvk_chess_core::fen::Fen;

    use crate::ReferenceEngine;

    const REFERENCE_ENGINE: ReferenceEngine = ReferenceEngine::new(r"C:\Users\Marvin\Desktop\stockfish_15_win_x64_avx2\stockfish_15_x64_avx2.exe");

    #[test]
    #[ignore]
    fn with_reference_engine() {
        compare_perft("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -", 4);
    }

    #[test]
    #[ignore]
    fn print_moves() {
        let mut bitboard = Bitboard::new(&Fen::new("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap());

        let mut moves = Vec::new();
        bitboard.generate_pseudo_legal_moves_with_buffer(&mut moves);

        for x in moves {
            bitboard.make(x);

            if bitboard.is_valid() {
                move_to_san(&x);
                println!("{}", bitboard);
            }

            bitboard.unmake(x);
        }
    }

    fn compare_perft(fen_str: &str, depth: usize) {
        if depth == 0 {
            println!("EXHAUSTED DEPTH");
            return;
        }

        let fen = &Fen::new(fen_str).unwrap();
        let mut bitboard = Bitboard::new(fen);

        let moves = bitboard.perft(depth);
        let actual: HashSet<(String, u64)> = HashSet::from_iter(moves.iter().map(|t| (move_to_san(&t.0), t.1)));
        let expected: HashSet<(String, u64)> = HashSet::from_iter(REFERENCE_ENGINE.perft(fen, depth));

        let actual_moves = actual.iter().map(|t| t.0.clone()).collect::<HashSet<_>>();
        let expected_moves = expected.iter().map(|t| t.0.clone()).collect::<HashSet<_>>();

        let excess = actual_moves.difference(&expected_moves).cloned().collect::<Vec<_>>();
        let missing = expected_moves.difference(&actual_moves).cloned().collect::<Vec<_>>();

        // println!("excess: {:?}", excess);
        // println!("missing: {:?}", missing);

        let has_excess = !excess.is_empty();
        let has_missing = !missing.is_empty();

        println!("FEN: {:?}", fen);

        if has_excess {
            println!("EXCESS:");
            for x in excess.iter() {
                println!("{}", &moves.iter().find(|&mv| &move_to_san(&mv.0) == x).unwrap().0);
            }
        }
        if has_missing {
            println!("MISSING:");
            for x in missing.iter() {
                println!("{}", &moves.iter().find(|&mv| &move_to_san(&mv.0) == x).unwrap().0);
            }
        }

        let not_wrong_count = excess.iter().chain(missing.iter()).cloned().collect::<Vec<_>>();
        let wrong_count = actual.difference(&expected).cloned().filter(|t| !not_wrong_count.contains(&t.0)).collect::<Vec<_>>();
        let has_wrong_count = !wrong_count.is_empty();

        // println!("wrong_count: {:?}", wrong_count);
        // println!("actual: {:?}", actual);
        // println!("expected: {:?}", expected);

        if has_wrong_count {
            println!("WRONG_COUNT:");
            for (mv, _) in wrong_count.iter() {
                let actual = actual.iter().find(|it| &it.0 == mv).unwrap();
                let expected = expected.iter().find(|it| &it.0 == mv).unwrap();

                println!("{} is {}, but should be {}", mv, actual.1, expected.1);
            }


            let string = &wrong_count.first().unwrap().0;
            let option = &moves.iter().find(|&mv| &move_to_san(&mv.0) == string).unwrap().0;


            println!("Going deeper into {}: ", move_to_san(option));
            println!("{}", bitboard);
            bitboard.make(*option);
            println!("{}", bitboard);

            let deep_fen = bitboard.fen();

            println!("{}", "-".repeat(100));
            compare_perft(&deep_fen.fen, depth - 1);
        }

        if has_excess || has_missing || has_wrong_count {
            panic!();
        }
    }

    #[test]
    #[ignore]
    fn simple() {
        let fen = Fen::new("r3k2r/p1ppqpb1/bnN1pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 1 1").unwrap();
        Bitboard::new(&fen).generate_pseudo_legal_moves_with_buffer(&mut Vec::new());
    }
}

#[cfg(test)]
mod perft {
    use std::usize;
    use std::time::SystemTime;

    use marvk_chess_board::board::{Bitboard, Move};
    use marvk_chess_core::fen::Fen;

    use crate::{expect, PerftResult};

    const LIMIT: u64 = 200_000_000;

    #[test]
    fn run_all() {
        perft1();

        println!("a");

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

        println!("{:?}", start.elapsed());
    }

    #[test]
    fn perft1() {
        run_perft(
            &"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &[
                expect(20),
                expect(400),
                expect(8_902),
                expect(197_281),
                expect(4_865_609),
                expect(119_060_324),
            ],
        )
    }

    #[test]
    fn perft2() {
        run_perft(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
            &[
                expect(48),
                expect(2_039),
                expect(97_862),
                expect(4_085_603),
                expect(193_690_690),
            ],
        )
    }

    #[test]
    fn perft3() {
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
        )
    }

    #[test]
    fn perft4() {
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
        )
    }

    #[test]
    fn perft5() {
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
        )
    }

    #[test]
    fn perft6() {
        run_perft(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            &[
                expect(44),
                expect(1_486),
                expect(62_379),
                expect(2_103_487),
                expect(89_941_194),
            ],
        )
    }

    #[test]
    fn perft7() {
        run_perft(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            &[
                expect(46),
                expect(2_079),
                expect(89_890),
                expect(3_894_594),
                expect(164_075_551),
            ],
        )
    }

    fn run_perft(fen_string: &str, expect: &[PerftResult]) {
        let fen = Fen::new(fen_string).unwrap();
        let mut board = Bitboard::new(&fen);

        let n = expect.iter().filter(|result| result.nodes < LIMIT).count();
        let actual =
            (1..=n)
                .into_iter()
                .map(|index| {
                    let mut result = PerftResult::new();
                    _run_perft_recursive(&mut board, &mut result, &mut Vec::new(), index);
                    result
                })
                .collect::<Vec<_>>();

        assert_eq!(actual, expect.iter().cloned().take(n).collect::<Vec<_>>(), "Failed for {}", fen_string);
    }

    fn _run_perft_recursive(board: &mut Bitboard, result: &mut PerftResult, buffer: &mut Vec<Move>, current_depth: usize) {
        if current_depth == 0 {
            result.nodes += 1;
            return;
        }

        board.generate_pseudo_legal_moves_with_buffer(buffer);

        let mut next_buffer = Vec::new();
        for mv in buffer {
            board.make(*mv);

            if board.is_valid() {
                _run_perft_recursive(board, result, &mut next_buffer, current_depth - 1);
                next_buffer.clear();
            }

            board.unmake(*mv);
        }
    }
}
