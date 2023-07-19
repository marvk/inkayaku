use std::io;

use async_stream::stream;
use futures::pin_mut;
use futures_util::{AsyncBufReadExt, StreamExt};
use futures_util::AsyncReadExt;
use futures_util::Stream;
use serde_json::Value;
use surf::{Client, Request, RequestBuilder, Response, StatusCode};
use surf::http::Method;

use crate::api::bot_event_response::BotEvent;
use crate::api::bot_game_state_response::BotGameState;

pub mod response;
pub mod bot_event_response;
pub mod bot_game_state_response;
pub mod request;

pub struct SurfWebClient {
    token: String,
    client: Client,
}

impl SurfWebClient {
    pub fn new(token: &str, client: Client) -> Self {
        Self { token:token.to_string(), client }
    }
}

#[derive(Debug)]
pub enum RequestError {
    SurfRequestError(surf::Error),
    SurfReadError(io::Error),
    SerdeParseError(serde_json::Error),
    SurfRequestErrorWithStatusCode(StatusCode),
}

impl SurfWebClient {
    fn request_builder(&self, url: &str, method: Method) -> RequestBuilder {
        self.client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.token))
    }

    async fn send_request(&self, request: Request) -> Result<Response, RequestError> {
        self.client.send(request).await.map_err(RequestError::SurfRequestError)
    }

    pub async fn stream(&self, url: &str) -> Result<impl Stream<Item=String> + '_, RequestError> {
        let request = self.request_builder(url, Method::Get).build();
        println!("{}", request.url());

        let mut response = self.send_request(request).await?;

        let status = response.status();

        if !status.is_success() {
            Err(RequestError::SurfRequestErrorWithStatusCode(status))
        } else {
            let s = stream! {

                loop {
                    let mut buf = String::new();

                    response.read_line(&mut buf).await.unwrap();

                    if buf.trim().is_empty() {
                        continue;
                    }

                    yield buf;
                }
            };
            Ok(s)
        }
    }

    async fn get(&self, url: &str) -> Result<String, RequestError> {
        let request = self.request_builder(url, Method::Get).build();
        println!("{}", request.url());

        let mut response = self.send_request(request).await?;

        let status = response.status();

        if !status.is_success() {
            Err(RequestError::SurfRequestErrorWithStatusCode(status))
        } else {
            let mut buf = String::new();
            response.read_to_string(&mut buf).await.map_err(RequestError::SurfReadError)?;

            Ok(buf)
        }
    }

    async fn post(&self, url: &str, body: Option<&Value>) -> Result<(), RequestError> {
        let request_builder = self.request_builder(url, Method::Post);
        let request = if let Some(body) = body {
            request_builder.body_string(serde_json::to_string(body).unwrap()).build()
        } else {
            request_builder.build()
        };
        println!("{}", request.url());

        let response = self.send_request(request).await?;

        let status = response.status();

        if !status.is_success() {
            Err(RequestError::SurfRequestErrorWithStatusCode(status))
        } else {
            Ok(())
        }
    }
}

pub struct BotApi {
    client: SurfWebClient,
}

impl BotApi {
    pub fn new(client: SurfWebClient) -> Self {
        Self { client }
    }
}

/// Bot operations
impl BotApi {
    /// Stream incoming events
    /// https://lichess.org/api#tag/Bot/operation/apiStreamEvent
    pub async fn stream_incoming_events(&self) -> Result<impl Stream<Item=BotEvent> + '_, RequestError> {
        Ok(stream! {
            let result = self.client
                .stream("/api/stream/event")
                .await
                .unwrap();

            pin_mut!(result);

            while let Some(s) = result.next().await {
                println!("\n{}\n", s);
                yield serde_json::from_str(&s).unwrap();
            }
        })
    }

    /// Get online bots
    /// https://lichess.org/api#tag/Bot/operation/apiBotOnline
    pub async fn get_online_bots(&self) -> Result<Vec<Value>, RequestError> {
        self
            .client
            .get("/api/bot/online")
            .await?
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()
            .map_err(RequestError::SerdeParseError)
    }

    /// Stream Bot game state
    /// https://lichess.org/api#tag/Bot/operation/botGameStream
    pub async fn stream_bot_game_state(&self, game_id: &str) -> Result<impl Stream<Item=BotGameState> + '_, RequestError> {
        let url = format!("api/bot/game/stream/{}", game_id);

        Ok(stream! {
            let result = self.client
                .stream(&url)
                .await
                .unwrap();

            pin_mut!(result);

            while let Some(s) = result.next().await {
                println!("\n{}\n", s);
                yield serde_json::from_str(&s).unwrap();
            }
        })
    }

    /// Make a Bot move
    /// https://lichess.org/api#tag/Bot/operation/botGameMove
    pub async fn post_bot_move(&self, game_id: &str, uci_move: &str, offering_draw: bool) -> Result<(), RequestError> {
        if offering_draw {
            panic!();
        }
        let url = format!("/api/bot/game/{}/move/{}", game_id, uci_move);
        self.client.post(&url, None).await
    }

    /// Write in the chat
    /// https://lichess.org/api#tag/Bot/operation/botGameChat
    pub async fn post_chat_message(&self) {
        todo!();
    }

    /// Fetch the game chat
    /// https://lichess.org/api#tag/Bot/operation/botGameChatGet
    pub async fn get_game_chat(&self) {
        todo!();
    }

    /// Abort a game
    /// https://lichess.org/api#tag/Bot/operation/botGameAbort
    pub async fn post_abort_game(&self) {
        todo!();
    }

    /// Resign a game
    /// https://lichess.org/api#tag/Bot/operation/botGameResign
    pub async fn post_resign_game(&self) {
        todo!();
    }
}

/// Challenges operations
impl BotApi {
    /// List your challenges
    /// https://lichess.org/api#tag/Challenges/operation/challengeList
    pub async fn get_challenges(&self) {
        todo!();
    }

    /// Create a challenge
    /// https://lichess.org/api#tag/Challenges/operation/challengeCreate
    pub async fn post_create_challenge(&self) {
        todo!();
    }

    /// Accept a challenge
    /// https://lichess.org/api#tag/Challenges/operation/challengeAccept
    pub async fn post_accept_challenge(&self, challenge_id: &str) -> Result<(), RequestError> {
        let url = format!("/api/challenge/{}/accept", challenge_id);
        self.client.post(&url, None).await
    }

    /// Decline a challenge
    /// https://lichess.org/api#tag/Challenges/operation/challengeDecline
    pub async fn post_decline_challenge(&self, challenge_id: &str) -> Result<(), RequestError> {
        let url = format!("/api/challenge/{}/decline", challenge_id);
        self.client.post(&url, None).await
    }

    /// Cancel a challenge
    /// https://lichess.org/api#tag/Challenges/operation/challengeCancel
    pub async fn post_cancel_challenge(&self, challenge_id: &str) -> Result<(), RequestError> {
        let url = format!("/api/challenge/{}/cancel", challenge_id);
        self.client.post(&url, None).await
    }
}
