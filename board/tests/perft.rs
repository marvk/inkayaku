extern crate core;

use std::fmt::{Debug, Formatter};
use std::io::Read;
use std::process::{Child, Command, Stdio};
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

    use marvk_chess_board::{move_to_san_reduced, occupancy_to_string};
    use marvk_chess_board::board::{Bitboard, Move};
    use marvk_chess_core::fen::Fen;

    use crate::ReferenceEngine;

    const REFERENCE_ENGINE: ReferenceEngine = ReferenceEngine::new(r"C:\Users\Marvin\Desktop\stockfish_15_win_x64_avx2\stockfish_15_x64_avx2.exe");

    #[test]
    #[ignore]
    fn with_reference_engine() {
        compare_perft("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 4);
    }

    #[test]
    #[ignore]
    fn print_moves() {
        let mut bitboard = Bitboard::new(&Fen::new("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap());

        for x in bitboard.generate_pseudo_legal_moves() {
            bitboard.make(Move(x.0));

            if bitboard.is_valid() {
                move_to_san_reduced(&x);
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
        let actual: HashSet<(String, u64)> = HashSet::from_iter(moves.iter().map(|t| (move_to_san_reduced(&t.0), t.1)));
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
                println!("{}", &moves.iter().find(|&mv| &move_to_san_reduced(&mv.0) == x).unwrap().0);
            }
        }
        if has_missing {
            println!("MISSING:");
            for x in missing.iter() {
                println!("{}", &moves.iter().find(|&mv| &move_to_san_reduced(&mv.0) == x).unwrap().0);
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
            let option = &moves.iter().find(|&mv| &move_to_san_reduced(&mv.0) == string).unwrap().0;


            println!("Going deeper into {}: ", move_to_san_reduced(option));
            println!("{}", bitboard);
            bitboard.make(Move(option.0));
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
        Bitboard::new(&fen).generate_pseudo_legal_moves();
    }

    #[test]
    #[ignore]
    fn asd() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";

        let mut bitboard = Bitboard::new(&Fen::new(fen).unwrap());

        let x = bitboard.generate_pseudo_legal_moves().into_iter().find(|mv| move_to_san_reduced(mv) == "e5c6").unwrap();

        println!("{}", x);

        bitboard.make(x);

        println!("{}", bitboard);

        let mut count = 0;
        let mut expect: HashSet<&str> = ["b4b3", "e6e5", "g6g5", "d7d6", "h3g2", "e6d5", "d7c6", "b4c3", "b6a4", "b6c4", "b6d5", "b6c8", "f6e4", "f6g4", "f6d5", "f6h5", "f6h7", "f6g8", "a6e2", "a6d3", "a6c4", "a6b5", "a6b7", "a6c8", "g7h6", "g7f8", "a8b8", "a8c8", "a8d8", "h8h4", "h8h5", "h8h6", "h8h7", "h8f8", "h8g8", "e7c5", "e7d6", "e7d8", "e7f8", "e8g8", "e8f8", ].try_into().unwrap();

        for x in bitboard.generate_pseudo_legal_moves() {
            bitboard.make(Move(x.0));

            if bitboard.is_valid() {
                count += 1;

                let string = move_to_san_reduced(&x);
                if !expect.remove(string.as_str()) {
                    println!("NOT FOUND: {}", string);
                } else {
                    println!("{}", string);
                }
            } else {
                println!("INVALID: {}", move_to_san_reduced(&x));
            }
            bitboard.unmake(x);
        }

        println!("{:?}", expect);
        println!("cnt. {}", count);
    }


    #[test]
    #[ignore]
    fn make_issue() {
        let fen = &Fen::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap();
        let mut board = Bitboard::new(fen);
        let original = Bitboard::new(fen);

        println!("{}", occupancy_to_string(64739317792112640));
        println!("{}", occupancy_to_string(65020788473856000));

        let mut count = 0;

        for x in board.generate_pseudo_legal_moves() {
            board.make(Move(x.0));
            if board.is_valid() {
                println!("{}", move_to_san_reduced(&x));
                count += 1;
            }
            board.unmake(Move(x.0));

            println!();
            println!("{}", occupancy_to_string(board.white.pawns));
            println!("{}", occupancy_to_string(original.white.pawns));
            println!("----");
            assert_eq!(board, original, "Issue with move {}", move_to_san_reduced(&x));
        }
        println!("{}", count);
    }

    #[test]
    #[ignore]
    fn perft() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";

        let mut expect: HashSet<&str> = ["f4f3", "d6d5", "c7c6", "c7c5", "h5b5", "h5c5", "h5d5", "h5e5", "h5f5", "h5g5", "h5h6", "h5h7", "h5h8", "h4g3", "h4h3", "h4g4", "h4g5", ].try_into().unwrap();

        let mut board = Bitboard::new(&Fen::new(fen).unwrap());

        println!("{}", board);

        let mut outer_count = 0;
        for mv in board.generate_pseudo_legal_moves() {
            board.make(Move(mv.0));


            if board.is_valid() {
                outer_count += 1;

                let mut count = 0;
                let moves2 = board.generate_pseudo_legal_moves();
                for mv2 in moves2 {
                    board.make(Move(mv2.0));

                    if board.is_valid() {
                        count += 1;
                    }

                    board.unmake(mv2);
                }
                println!("{}: {}", move_to_san_reduced(&mv), count);


                // let string = move_to_san_reduced(&mv);
                // println!("{}", string);
                // if !expect.remove(string.as_str()) {
                //     println!("NOT FOUND: {}", string);
                // }
            }

            board.unmake(mv);
        }

        println!("num: {}", outer_count);
        println!("{:?}", expect);
    }
}

#[cfg(test)]
mod perft {
    use std::collections::HashSet;
    use std::time::SystemTime;
    use std::usize;

    use marvk_chess_board::{move_to_san_reduced, occupancy_to_string};
    use marvk_chess_board::board::{Bitboard, Move};
    use marvk_chess_core::fen::Fen;

    use crate::{expect, PerftResult};

    const LIMIT: u64 = 5_000_000_000;

    #[test]
    fn run_all() {
        perft1();

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
                    _run_perft_recursive(&mut board, &mut result, index);
                    result
                })
                .collect::<Vec<_>>();

        assert_eq!(actual, expect.iter().cloned().take(n).collect::<Vec<_>>());
    }

    fn _run_perft_recursive(board: &mut Bitboard, result: &mut PerftResult, current_depth: usize) {
        if current_depth == 0 {
            result.nodes += 1;
            return;
        }

        let moves = board.generate_pseudo_legal_moves();

        for mv in moves.into_iter() {
            board.make(Move(mv.0));

            if board.is_valid() {
                _run_perft_recursive(board, result, current_depth - 1);
            }

            board.unmake(Move(mv.0));
        }
    }
}
