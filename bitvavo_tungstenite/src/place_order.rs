use crate::rug_float_serde::FloatWrapper;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::{Error, Message};

pub async fn place_buy_limit_order(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    market: &str,
    quantity: FloatWrapper,
    price: FloatWrapper,
) -> Result<(), Error> {
    let order_message = json!({
        "action": "placeOrder",
        "market": market,
        "side": "buy",
        "orderType": "limit",
        "amount": quantity.to_string(),
        "price": price.to_string(),
    });

    write.send(Message::Text(order_message.to_string())).await
}

pub async fn place_sell_limit_order(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    market: &str,
    quantity: FloatWrapper,
    price: FloatWrapper,
) -> Result<(), Error> {
    let order_message = json!({
        "action": "placeOrder",
        "market": market,
        "side": "sell",
        "orderType": "limit",
        "amount": quantity.to_string(),
        "price": price.to_string(),
    });

    write.send(Message::Text(order_message.to_string())).await
}

pub async fn place_buy_market_order(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    market: &str,
    quantity: FloatWrapper,
) -> Result<(), Error> {
    let order_message = json!({
        "action": "placeOrder",
        "market": market,
        "side": "buy",
        "orderType": "market",
        "amount": quantity.to_string(),
    });

    write.send(Message::Text(order_message.to_string())).await
}

pub async fn place_sell_market_order(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    market: &str,
    quantity: FloatWrapper,
) -> Result<(), Error> {
    let order_message = json!({
        "action": "placeOrder",
        "market": market,
        "side": "sell",
        "orderType": "market",
        "amount": quantity.to_string(),
    });

    write.send(Message::Text(order_message.to_string())).await
}
