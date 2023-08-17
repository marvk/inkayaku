use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::api::response::{Color, ColorChoice, GameStatusKey, PerfKey, SpeedKey, VariantFull, VariantKey};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum BotEvent {
    GameStart { game: GameEventInfo },
    GameFinish { game: GameEventInfo },
    // Compat isn't being sent when the message arrived in the lila queue before the stream is started
    Challenge { challenge: ChallengeEventInfo, compat: Option<Compat> },
    ChallengeDeclined { challenge: ChallengeEventInfo },
    ChallengeCanceled { challenge: ChallengeEventInfo },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameEventInfo {
    pub full_id: String,
    pub game_id: String,
    pub fen: String,
    pub color: Color,
    pub last_move: String,
    pub source: GameEventSource,
    pub status: GameEventStatus,
    pub variant: GameEventVariant,
    pub speed: GameEventSpeedKey,
    pub perf: PerfKey,
    pub rated: bool,
    pub has_moved: bool,
    pub opponent: GameEventOpponent,
    pub seconds_left: Option<u32>,
    pub tournament_id: Option<String>,
    pub swiss_id: Option<String>,
    pub orientation: Option<Color>,
    pub winner: Option<Color>,
    pub rating_diff: Option<i32>,
    pub compat: Option<Compat>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum GameEventSpeedKey {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameEventVariant {
    pub key: VariantKey,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameEventOpponent {
    pub id: String,
    pub username: String,
    pub rating: Option<u32>,
    pub rating_diff: Option<i32>,
    pub ai: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GameEventSource {
    Lobby,
    Friend,
    Ai,
    Api,
    Arena,
    Position,
    Import,
    ImportLive,
    Simul,
    Relay,
    Pool,
    Swiss,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeEventInfo {
    pub id: String,
    pub url: String,
    pub status: ChallengeEventStatusKey,
    pub challenger: Option<Challenger>,
    pub dest_user: Option<Challenger>,
    pub variant: VariantFull,
    pub rated: bool,
    pub speed: SpeedKey,
    pub time_control: ChallengeEventTimeControl,
    pub color: ColorChoice,
    pub final_color: Color,
    pub perf: ChallengeEventPerf,
    pub rematch_of: Option<String>,
    pub direction: Option<ChallengeEventDirection>,
    pub initial_fen: Option<String>,
    pub decline_reason: Option<ChallengeEventDeclineReason>,
    #[serde(deserialize_with = "from_csv")]
    #[serde(default)]
    pub rules: Vec<ChallengeEventRule>,
}

fn from_csv<'de, D>(deserializer: D) -> Result<Vec<ChallengeEventRule>, D::Error>
    where
        D: Deserializer<'de>
{
    let string: &str = Deserialize::deserialize(deserializer)?;
    let result = string.split(',').map(|s| ChallengeEventRule::from_str(s).unwrap()).collect();
    Ok(result)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ChallengeEventTimeControl {
    #[serde(rename_all = "camelCase")]
    Clock {
        limit: u32,
        increment: u32,
        show: String,
    },
    #[serde(rename_all = "camelCase")]
    Correspondence {
        days_per_turn: u32,
    },
    #[serde(rename_all = "camelCase")]
    Unlimited,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeEventStatusKey {
    Created,
    Offline,
    Canceled,
    Declined,
    Accepted,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Challenger {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub rating: u32,
    pub provisional: Option<bool>,
    pub patron: Option<bool>,
    pub online: Option<bool>,
    pub lag: Option<u32>,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeEventPerf {
    pub icon: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeEventDirection {
    In,
    Out,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeEventDeclineReason {
    Generic,
    Later,
    TooFast,
    TooSlow,
    TimeControl,
    Rated,
    Casual,
    Standard,
    Variant,
    NoBot,
    OnlyBot,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeEventRule {
    NoAbort,
    NoRematch,
    NoGiveTime,
    NoClaimWin,
    NoEarlyDraw,
}

impl FromStr for ChallengeEventRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "noabort" => Ok(Self::NoAbort),
            "norematch" => Ok(Self::NoRematch),
            "nogivetime" => Ok(Self::NoGiveTime),
            "noclaimwin" => Ok(Self::NoClaimWin),
            "noearlydraw" => Ok(Self::NoEarlyDraw),
            _ => Err(())
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Compat {
    pub bot: bool,
    pub board: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameEventStatus {
    id: u32,
    name: GameStatusKey,
}
