use std::cell::{RefCell, RefMut};
use std::str::FromStr;

use std::sync::{Arc};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use futures::executor::block_on;
use futures::pin_mut;
use futures_util::StreamExt;

use inkayaku_board::Bitboard;
use inkayaku_core::constants::Color;
use inkayaku_core::fen::Fen;
use inkayaku_engine_core::engine::Engine;
use inkayaku_lichess_api::api::bot_event_response::ChallengeEventDeclineReason;
use inkayaku_lichess_api::api::bot_game_state_response::{BotGameState, Clock, GameStateHolder};
use inkayaku_lichess_api::api::BotApi;
use inkayaku_lichess_api::api::response::{GameStatusKey, SpeedKey, VariantFull, VariantKey};
use inkayaku_uci::{UciEngine, Go, UciCommand, UciMove, UciTxCommand};
use inkayaku_uci::command::CommandUciTx;


pub struct GameThread {
    bot_id: String,
    game_id: String,
    api: Arc<BotApi>,
    engine: RefCell<Engine<CommandUciTx>>,
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
        println!("START GAME THREAD");
        let stream = self.api.stream_bot_game_state(&self.game_id).await.unwrap();
        pin_mut!(stream);

        while let Some(state) = stream.next().await {
            match state {
                BotGameState::GameFull { state, id, variant, speed, perf, rated, created_at, white, black, initial_fen, clock, days_per_turn, tournament_id } => {
                    let fen = Fen::from_str(&initial_fen).unwrap();

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
        if initial_fen.ne(&Fen::default()) || !matches!(variant.key, VariantKey::Standard) {
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
        let moves = state.moves.iter().map(|m| UciMove::from_str(m).unwrap()).collect();

        match state.status {
            GameStatusKey::Created | GameStatusKey::Started => {
                if self.is_my_turn(&moves) {
                    let fen = self.game_state.borrow().initial_fen().clone();
                    engine.accept(UciCommand::PositionFrom { fen, moves });
                    engine.accept(UciCommand::Go {
                        go: Go {
                            white_time: Some(Duration::from_millis(state.wtime as u64)),
                            black_time: Some(Duration::from_millis(state.btime as u64)),
                            white_increment: Some(Duration::from_millis(state.winc as u64)),
                            black_increment: Some(Duration::from_millis(state.binc as u64)),
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
        let mut bitboard: Bitboard = self.game_state.borrow().initial_fen.clone().unwrap().into();

        for mv in moves {
            bitboard.make_uci(&mv.to_string()).unwrap();
        }

        self.game_state.borrow().self_color().index == bitboard.turn
    }

    fn engine(&self) -> RefMut<Engine<CommandUciTx>> {
        self.engine.borrow_mut()
    }

    fn spawn_engine(api: Arc<BotApi>, game_id: &str) -> Engine<CommandUciTx> {
        let (tx, rx): (Sender<UciTxCommand>, _) = channel();
        Self::spawn_engine_rx_thread(rx, api, game_id);

        Engine::new(Arc::new(CommandUciTx::new(tx)), false)
    }

    fn spawn_engine_rx_thread(rx: Receiver<UciTxCommand>, api: Arc<BotApi>, game_id: &str) {
        let game_id = game_id.to_string();

        thread::spawn(move || {
            let send_uci_move = |uci_move: UciMove| {
                block_on(api.post_bot_move(&game_id, &uci_move.to_string(), false)).unwrap();
            };

            while let Ok(command) = rx.recv() {
                match command {
                    UciTxCommand::BestMove { best_move: Some(uci_move), .. } => {
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
