use serde::{Deserialize, Deserializer, Serialize};

use crate::api::response::{Color, GameStatusKey, SpeedKey, VariantFull};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum BotGameState {
    #[serde(rename_all = "camelCase")]
    GameFull {
        state: GameStateHolder,
        id: String,
        variant: VariantFull,
        speed: SpeedKey,
        perf: Perf,
        rated: bool,
        created_at: u64,
        white: Player,
        black: Player,
        initial_fen: String,
        clock: Option<Clock>,
        days_per_turn: Option<u32>,
        tournament_id: Option<String>,
    },
    GameState {
        #[serde(flatten)]
        state: GameStateHolder
    },
    ChatLine {
        room: Room,
        username: String,
        text: String,
    },
    OpponentGone {
        gone: bool,
        claim_win_in_seconds: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Room {
    Player,
    Spectator,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameStateHolder {
    #[serde(deserialize_with = "from_space_sv")]
    #[serde(default)]
    pub moves: Vec<String>,
    pub wtime: u32,
    pub btime: u32,
    pub winc: u32,
    pub binc: u32,
    pub status: GameStatusKey,
    pub wdraw: Option<bool>,
    pub bdraw: Option<bool>,
    pub wtakeback: Option<bool>,
    pub btakeback: Option<bool>,
    pub winner: Option<Color>,
    pub rematch: Option<String>,
}

fn from_space_sv<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>
{
    let string: &str = Deserialize::deserialize(deserializer)?;
    if string.trim().is_empty() {
        Ok(Vec::default())
    } else {
        let result = string.split(' ').map(&str::to_string).collect();
        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Perf {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub ai_level: Option<u32>,
    pub id: String,
    pub name: Option<String>,
    pub title: Option<String>,
    pub rating: Option<u32>,
    pub provisional: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Clock {
    pub initial: u32,
    pub increment: u32,
}
