use std::fs;

use futures::pin_mut;
use futures_util::StreamExt;
use surf::{Client, Url};

use inkayaku_lichess_api::api::{BotApi, SurfWebClient};
use inkayaku_lichess_api::api::bot_event_response::BotEvent;

use crate::bot::GameThread;

mod bot;

#[tokio::main]
async fn main() {
    let token = fs::read_to_string("token").unwrap();


    let client = create_client();
    let swc = SurfWebClient::new(&token, client);
    let api = BotApi::new(swc);

    let event_stream = api.stream_incoming_events().await.unwrap();

    pin_mut!(event_stream);

    while let Some(value) = event_stream.next().await {
        println!("RECEIVED EVENT {:?}", value);

        match value {
            BotEvent::Challenge { challenge, compat: _compat } => {
                let id = challenge.id;
                api.post_accept_challenge(&id).await.unwrap_or_default();
            }
            BotEvent::GameStart { game } => {
                let thread = GameThread::new("kingsgambot", &game.game_id, BotApi::new(SurfWebClient::new(&token, create_client())));

                tokio::spawn(thread.start());
            }
            _ => {}
        }

        println!("HANDLED EVENT");
    }

    // println!("{:?}", x);
}

fn create_client() -> Client {
    surf::Config::new()
        .set_timeout(None)
        .set_base_url(Url::parse("https://lichess.org/").unwrap())
        .try_into()
        .unwrap()
}
