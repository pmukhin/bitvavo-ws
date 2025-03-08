use bitvavo_ws_rust::bitvavo::{Bitvavo, SubscriptionBuilder};
use bitvavo_ws_rust::decode::decode_event;
use bitvavo_ws_rust::event::{AuthRequest, BitvavoEvent};
use bitvavo_ws_rust::local_book::LocalBook;
use clap::Parser;
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio_tungstenite::connect_async;
use tungstenite::client::IntoClientRequest;

#[derive(Parser, Debug)]
#[clap(
    name = "bitvavo-ws",
    version = "0.0.1",
    author = "Pavel Mukhin <your.email@example.com>"
)]
struct Config {
    #[clap(short('k'), long, value_name = "API_KEY", required = true)]
    pub api_key: String,

    #[clap(short('s'), long, value_name = "API_SECRET", required = true)]
    pub api_secret: String,

    #[clap(short('u'), long, value_name = "WS_URL", required = true)]
    pub ws_url: String,

    #[clap(short('b'), long, value_name = "BASE_ASSET", required = true)]
    pub base_asset: String,

    #[clap(short('q'), long, value_name = "QUOTE_ASSET", required = true)]
    pub quote_asset: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Config::parse();
    let market_symbol = format!("{}-{}", &config.base_asset, &config.quote_asset);

    let url = config.ws_url.into_client_request().unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    let (write, mut read) = ws_stream.split();

    // wrapping write Sink into the Bitvavo struct to send commands
    let mut bitvavo = Bitvavo::wrap(write);

    bitvavo
        .authenticate(AuthRequest::make(&config.api_key, &config.api_secret))
        .await
        .expect("failed request authentication");

    // let's wait authentication is complete
    let msg = read
        .next()
        .await
        .expect("error authentication response reading message");

    log::info!("authentication succeeded: {}", msg.unwrap());

    // now we can call actions and receive events/updates
    let mut local_book = LocalBook::default();
    let mut balances = HashMap::default();

    bitvavo
        .get_balances()
        .await
        .expect("failed to request balances");

    bitvavo
        .get_book(&market_symbol)
        .await
        .expect("failed to get book");

    let sb = SubscriptionBuilder::default()
        .with_market(market_symbol)
        .with_ticker()
        .with_account()
        .with_trades()
        .with_candles("1m");

    bitvavo.subscribe(sb).await.expect("failed to subscribe");

    loop {
        let msg = read.next().await;
        match msg {
            Some(Ok(tungstenite::Message::Close(_))) => {
                log::error!("server closed the connection");
                break;
            }
            Some(Ok(tungstenite::Message::Text(text))) => match decode_event(&text) {
                Err(e) => log::error!("error decoding event: {:?}", e),
                Ok(BitvavoEvent::Subscribed) => log::debug!("successfully subscribed"),
                // control events
                Ok(BitvavoEvent::Book(book)) => local_book.ingest_book(book),
                Ok(BitvavoEvent::Candle(_e)) => {}
                Ok(BitvavoEvent::Trade(_e)) => {}
                Ok(BitvavoEvent::Markets(_markets)) => {}
                Ok(BitvavoEvent::TickerBook(_e)) => {}
                Ok(BitvavoEvent::Ticker24h(_e)) => {}
                Ok(BitvavoEvent::Balances(b)) => balances = b,
                Ok(BitvavoEvent::Ticker(ticker)) => local_book.ingest_ticker(ticker),
            },
            Some(Ok(tungstenite::Message::Ping(m))) => {
                bitvavo.pong(m).await.expect("failed to pong")
            }
            // etc
            _ => {}
        }
        log::info!(
            "local book top: {:?} : {:?}, spread: {}%",
            local_book.top_bid_or_default(),
            local_book.top_ask_or_default(),
            local_book.real_spread_or_default().format_as_percentage(),
        );
    }
}
