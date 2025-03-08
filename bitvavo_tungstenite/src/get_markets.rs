use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::{Display, Formatter};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    status: String,
    base: String,
    quote: String,
    market: String,
    price_precision: Option<u32>,
    min_order_in_quote_asset: Option<String>,
    min_order_in_base_asset: Option<String>,
    order_types: Option<Vec<String>>,
}

impl Display for Market {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Market [ \
                status: {}, \
                base: {}, \
                quote: {}, \
                market: {}, \
                price_precision: {}, \
                min_order_in_quote_asset: {}, \
                min_order_in_base_asset: {}, \
                order_types: {:?}, \
            ]",
            self.status,
            self.base,
            self.quote,
            self.market,
            self.price_precision
                .map(|o| o.to_string())
                .get_or_insert(String::from("<empty>")),
            self.min_order_in_quote_asset
                .as_ref()
                .map(|o| o.to_string())
                .get_or_insert(String::from("<empty>")),
            self.min_order_in_base_asset
                .as_ref()
                .map(|o| o.to_string())
                .get_or_insert(String::from("<empty>")),
            self.order_types.as_ref().unwrap_or(&Vec::<String>::new()),
        )
    }
}

// MarketsResponse and Markets
#[derive(Serialize, Deserialize, Debug)]
pub struct MarketsResponse {
    action: String,
    pub response: Vec<Market>,
}

impl Display for MarketsResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Markets: {:?}", self.response,)
    }
}

pub async fn get_markets(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) -> Result<(), tungstenite::Error> {
    let markets_message = json!({
        "action": "getMarkets",
    });
    write.send(Message::Text(markets_message.to_string().into())).await
}
