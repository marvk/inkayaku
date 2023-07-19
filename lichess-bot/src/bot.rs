use std::cell::{Cell, RefCell, RefMut};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use futures::executor::block_on;
use futures::pin_mut;
use futures_util::StreamExt;

use marvk_chess_board::board::Bitboard;
use marvk_chess_core::constants::color::Color;
use marvk_chess_core::fen::{Fen, FEN_STARTPOS};
use marvk_chess_engine_lib::inkayaku::Inkayaku;
use marvk_chess_lichess_api::api::bot_event_response::ChallengeEventDeclineReason;
use marvk_chess_lichess_api::api::bot_game_state_response::{BotGameState, Clock, GameStateHolder};
use marvk_chess_lichess_api::api::BotApi;
use marvk_chess_lichess_api::api::response::{GameStatusKey, SpeedKey, VariantFull, VariantKey};
use marvk_chess_uci::uci::{Engine, Go, Info, ProtectionMessage, UciCommand, UciMove, UciTx, UciTxCommand};
use marvk_chess_uci::uci::console::ConsoleUciTx;
use marvk_chess_uci::uci::message::MessageUciTx;

pub struct GameThread {
    bot_id: String,
    game_id: String,
    api: Arc<BotApi>,
    engine: RefCell<Inkayaku<MessageUciTx>>,
    game_state: RefCell<GameState>,
}

#[derive(Default)]
struct GameState {
    initial_fen: Option<Fen>,
    self_color: Option<Color>,
}

impl GameState {
    fn initial_fen(&self) -> &Fen {
        self.initial_fen.as_ref().unwrap()
    }

    fn self_color(&self) -> &Color {
        self.self_color.as_ref().unwrap()
    }
}

impl GameThread {
    pub fn new(bot_id: &str, game_id: &str, api: BotApi) -> Self {
        let api = Arc::new(api);
        let engine = Self::spawn_engine(api.clone(), game_id);

        Self { bot_id: bot_id.to_string(), game_id: game_id.to_string(), api, engine: RefCell::new(engine), game_state: RefCell::new(GameState::default()) }
    }

    pub async fn start(self) {
        println!("start");
        let stream = self.api.stream_bot_game_state(&self.game_id).await.unwrap();
        pin_mut!(stream);

        dbg!("we got a stream");
        while let Some(state) = stream.next().await {
            dbg!(&state);

            match state {
                BotGameState::GameFull { state, id, variant, speed, perf, rated, created_at, white, black, initial_fen, clock, days_per_turn, tournament_id } => {
                    let fen = Fen::new(&initial_fen).unwrap();

                    self.game_state.borrow_mut().self_color = Some(if white.id == self.bot_id {
                        Color::WHITE
                    } else if black.id == self.bot_id {
                        Color::BLACK
                    } else {
                        panic!();
                    });

                    self.game_state.borrow_mut().initial_fen = Some(fen);
                    self.initialize_engine();
                    if !self.accept_state(state) {
                        return;
                    };
                }
                BotGameState::GameState { state, .. } => {
                    if !self.accept_state(state) {
                        return;
                    };
                }
                BotGameState::ChatLine { room, username, text } => {}
                BotGameState::OpponentGone { gone, claim_win_in_seconds } => {}
            }
        }
    }

    fn decide_accept(&self, variant: VariantFull, speed: SpeedKey, clock: Option<Clock>, initial_fen: &Fen) -> Option<ChallengeEventDeclineReason> {
        if initial_fen.ne(&FEN_STARTPOS) || !matches!(variant.key, VariantKey::Standard) {
            Some(ChallengeEventDeclineReason::Standard)
        } else {
            match speed {
                SpeedKey::Bullet => {
                    None
                }
                SpeedKey::UltraBullet | SpeedKey::Blitz | SpeedKey::Rapid | SpeedKey::Classical | SpeedKey::Correspondence => {
                    Some(ChallengeEventDeclineReason::Standard)
                }
            }
        }
    }

    fn initialize_engine(&self) {
        let mut engine = self.engine();
        engine.accept(UciCommand::UciNewGame);
    }

    fn accept_state(&self, state: GameStateHolder) -> bool {
        let mut engine = self.engine();
        let moves = state.moves.iter().map(|m| UciMove::parse(m).unwrap()).collect();

        match state.status {
            GameStatusKey::Created | GameStatusKey::Started => {
                if self.is_my_turn(&moves) {
                    let fen = self.game_state.borrow().initial_fen().clone();
                    engine.accept(UciCommand::PositionFrom { fen, moves });
                    engine.accept(UciCommand::Go {
                        go: Go {
                            ..Go::default()
                        }
                    });
                }
                true
            }
            _ => false,
        }
    }

    fn is_my_turn(&self, moves: &Vec<UciMove>) -> bool {
        let mut bitboard = Bitboard::new(&self.game_state.borrow().initial_fen.clone().unwrap());

        for mv in moves {
            bitboard.make_uci(&mv.to_string()).unwrap();
        }

        self.game_state.borrow().self_color().index == bitboard.turn
    }

    fn engine(&self) -> RefMut<Inkayaku<MessageUciTx>> {
        self.engine.borrow_mut()
    }

    fn spawn_engine(api: Arc<BotApi>, game_id: &str) -> Inkayaku<MessageUciTx> {
        let (tx, rx): (Sender<UciTxCommand>, _) = channel();
        Self::spawn_engine_rx_thread(rx, api, game_id);

        Inkayaku::new(Arc::new(MessageUciTx::new(Mutex::new(tx))))
    }

    fn spawn_engine_rx_thread(rx: Receiver<UciTxCommand>, api: Arc<BotApi>, game_id: &str) {
        let game_id = game_id.to_string();

        thread::spawn(move || {
            let send_uci_move = |uci_move: UciMove| {
                block_on(api.post_bot_move(&game_id, &uci_move.to_string(), false)).unwrap();
            };

            while let Ok(command) = rx.recv() {
                match command {
                    UciTxCommand::BestMove { uci_move } => {
                        if let Some(uci_move) = uci_move {
                            send_uci_move(uci_move);
                        }
                    }
                    UciTxCommand::BestMoveWithPonder { uci_move, .. } => {
                        send_uci_move(uci_move);
                    }
                    UciTxCommand::Info { info } => {
                        println!("{:?}", info);
                    }
                    _ => {}
                };
            }
        });
    }
}
