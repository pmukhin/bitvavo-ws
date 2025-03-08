use bitvavo_ws_rust::bitvavo_ws::decode_text;
use bitvavo_ws_rust::event::{AuthRequest, BitvavoEvent};
use bitvavo_ws_rust::get_balances::get_balances;
use bitvavo_ws_rust::get_book::get_book;
use bitvavo_ws_rust::local_book::LocalBook;
use bitvavo_ws_rust::subscribe::subscribe;
use clap::Parser;
use futures_util::SinkExt;
use futures_util::StreamExt;
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
    api_key: String,

    #[clap(short('s'), long, value_name = "API_SECRET", required = true)]
    api_secret: String,

    #[clap(short('u'), long, value_name = "WS_URL", required = true)]
    ws_url: String,

    #[clap(short('b'), long, value_name = "BASE_ASSET", required = true)]
    base_asset: String,

    #[clap(short('q'), long, value_name = "QUOTE_ASSET", required = true)]
    quote_asset: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Config::parse();

    let url = config.ws_url.into_client_request().unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let auth_req = AuthRequest::make(&config.api_key, &config.api_secret);
    let auth_req_str = serde_json::to_string(&auth_req).unwrap();

    write
        .send(tungstenite::Message::Text(auth_req_str.into()))
        .await
        .expect("failed to send auth request");

    let msg = read
        .next()
        .await
        .expect("error authentication response reading message");

    log::info!("authentication succeeded: {}", msg.unwrap());

    let mut local_book = LocalBook::default();

    let balances = get_balances(&mut write, &mut read)
        .await
        .expect("failed to get balances");
    let current_eur_balance: f64 = balances
        .get("EUR")
        .map(|x| x.available.parse::<f64>().unwrap())
        .unwrap_or(0.0);
    let current_base_balance: f64 = balances
        .get(&config.base_asset)
        .map(|x| x.available.parse::<f64>().unwrap())
        .unwrap_or(0.0);

    log::info!("current EUR balance: {}", current_eur_balance);
    log::info!(
        "current {} balance: {}",
        &config.base_asset,
        current_base_balance
    );

    let market_symbol = format!("{}-{}", &config.base_asset, &config.quote_asset);

    get_book(&mut write, &market_symbol)
        .await
        .expect("failed to get book");

    subscribe(&mut write, &market_symbol).await.expect("failed to subscribe");

    loop {
        let msg = read.next().await;
        match msg {
            None => log::debug!("no message received on time"),
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
                Ok(BitvavoEvent::Book(book)) => local_book.ingest_book(book),
                Ok(BitvavoEvent::Candle(_e)) => {}
                Ok(BitvavoEvent::Trade(_e)) => {}
                Ok(BitvavoEvent::Markets(_markets)) => {}
                Ok(BitvavoEvent::TickerBook(_e)) => {}
                Ok(BitvavoEvent::Ticker24h(_e)) => {}
                Ok(BitvavoEvent::Ticker(ticker)) => local_book.ingest_ticker(ticker),
            },
            Some(Ok(tungstenite::Message::Ping(m))) => {
                write
                    .send(tungstenite::Message::Pong(m))
                    .await
                    .expect("failed to send pong");
            }
            // etc
            Some(m) => {
                log::warn!("ignoring unexpected message: {:?}", m);
            }
        }
        log::info!(
            "local book top: {:?} : {:?}, spread: {}%",
            local_book.top_bid_or_default(),
            local_book.top_ask_or_default(),
            local_book.real_spread_or_default().format_as_percentage(),
        )
    }
}
