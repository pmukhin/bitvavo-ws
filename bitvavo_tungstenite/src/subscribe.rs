use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Error;

pub async fn subscribe(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
    market: &str,
) -> Result<(), Error> {
    let subscribe_message = json!({
        "action": "subscribe",
        "channels": [
            {
                "name": "ticker",
                "markets": [ market ],
            }, {
                "name": "candles",
                "interval": [ "1h" ],
                "markets": [ market ]
            }, /*{
                "name": "book",
                "markets": [ market ]
            },*/ {
                "name": "trades",
                "markets": [ market ]
            }, {
                "name": "account",
                "markets": [ market ],
            },
        ]
    });
    write
        .send(tungstenite::Message::Text(subscribe_message.to_string().into()))
        .await
}
