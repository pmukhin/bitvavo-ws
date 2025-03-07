mod bitvavo_ws;
mod book;
mod candle;
mod event;
mod markets;
mod rug_float_serde;
mod side;
mod sig;
mod trade;
mod get_balances;

use bitvavo_ws::decode_text;
use book::Book;
use clap::Parser;
use event::{AuthRequest, BitvavoEvent};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use futures_util::StreamExt;
use get_balances::get_balances;
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::client::IntoClientRequest;

#[derive(Parser, Debug)]
#[clap(name = "bitvavo-ws", version = "0.0.1", author = "Pavel Mukhin <your.email@example.com>")]
struct Config {
    #[clap(short('k'), long, value_name = "API_KEY", required = true)]
    api_key: String,

    #[clap(short('s'), long, value_name = "API_SECRET", required = true)]
    api_secret: String,

    #[clap(short('u'), long, value_name = "WS_URL", required = true)]
    ws_url: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Config::parse();
    let traded_base = "BTC";

    let url = config.ws_url.into_client_request().unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let auth_req = AuthRequest::make(&config.api_key, &config.api_secret);
    let auth_req_str = serde_json::to_string(&auth_req).unwrap();

    write
        .send(tungstenite::Message::Text(auth_req_str))
        .await
        .expect("failed to send auth request");

    let msg = read
        .next()
        .await
        .expect("error authentication response reading message");

    log::debug!("authentication succeeded: {}", msg.unwrap());

    let mut local_book = Arc::new(Book::default());

    let balances =
        get_balances(&mut write, &mut read).await.expect("failed to get balances");
    let current_eur_balance: f64 =
        balances.get("EUR").map(|x| x.available.parse::<f64>().unwrap()).unwrap_or(0.0);
    let current_base_balance: f64 =
        balances.get(traded_base).map(|x| x.available.parse::<f64>().unwrap()).unwrap_or(0.0);

    log::info!("current EUR balance: {}", current_eur_balance);
    log::info!("current {} balance: {}", traded_base, current_base_balance);

    subscribe(&mut write).await;

    loop {
        let msg = read.next().await;
        match msg {
            None => continue,
            Some(Err(e)) => {
                log::error!("error reading message: {:?}", e);
                continue;
            }
            Some(Ok(tungstenite::Message::Close(_))) => {
                log::error!("server closed the connection");
                break;
            }
            Some(Ok(tungstenite::Message::Text(text))) => match decode_text(&text) {
                Err(e) => log::error!("error decoding event: {:?}", e),
                Ok(BitvavoEvent::Subscribed) => log::debug!("successfully subscribed"),
                // control events
                Ok(BitvavoEvent::Book(book)) => { *Arc::get_mut(&mut local_book).unwrap() = book; }
                Ok(BitvavoEvent::Candle(_e)) => {}
                Ok(BitvavoEvent::Trade(_e)) => {}
                Ok(BitvavoEvent::MarketsResponse(_e)) => {}
                Ok(BitvavoEvent::TickerBook(_e)) => {}
                Ok(BitvavoEvent::Ticker24h(_e)) => {}
                Ok(BitvavoEvent::Ticker(_e)) => {}
            },
            // ping, etc
            Some(_) => { continue; }
        }
        log::info!("local book: {:?}", local_book);
    }
}

async fn subscribe(write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>) {
    let subscribe_message = json!({
        "action": "subscribe",
        "channels": [
            {
                "name": "ticker",
                "markets": [ "BTC-EUR" ],
            }, {
                "name": "candles",
                "interval": [ "1h" ],
                "markets": [ "BTC-EUR" ]
            }, {
                "name": "book",
                "markets": [ "BTC-EUR" ]
            }, {
                "name": "trades",
                "markets": [ "BTC-EUR" ]
            }
        ]
    });

    write
        .send(tungstenite::Message::Text(subscribe_message.to_string()))
        .await
        .expect("failed to subscribe to events");
}
