use crate::event::AuthRequest;
use crate::rug_float_serde::FloatWrapper;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Bytes;

#[derive(Debug, Default)]
pub struct SubscriptionBuilder {
    market: String,
    trades: bool,
    account: bool,
    book: bool,
    ticker: bool,
    candles: bool,
    candles_interval: &'static str,
}

impl SubscriptionBuilder {
    pub fn with_market(mut self, market: String) -> Self {
        self.market = market;
        self
    }

    pub fn with_trades(mut self) -> Self {
        self.trades = true;
        self
    }

    pub fn with_account(mut self) -> Self {
        self.account = true;
        self
    }

    pub fn with_book(mut self) -> Self {
        self.book = true;
        self
    }

    pub fn with_ticker(mut self) -> Self {
        self.ticker = true;
        self
    }

    pub fn with_candles(mut self, interval: &'static str) -> Self {
        self.candles = true;
        self.candles_interval = interval;
        self
    }
}

pub struct Bitvavo {
    stream: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
}

impl Bitvavo {
    pub fn wrap(
        write_stream: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
    ) -> Self {
        Bitvavo {
            stream: write_stream,
        }
    }

    pub async fn authenticate(&mut self, auth_req: AuthRequest) -> Result<(), tungstenite::Error> {
        let auth_req_str = serde_json::to_string(&auth_req).unwrap();
        self.stream
            .send(tungstenite::Message::Text(auth_req_str.into()))
            .await
    }

    pub async fn get_book(&mut self, market: &str) -> Result<(), tungstenite::Error> {
        let markets_message = json!({
            "action": "getBook",
            "market": market,
        });
        self.stream
            .send(tungstenite::Message::Text(
                markets_message.to_string().into(),
            ))
            .await
    }

    pub async fn subscribe(
        &mut self,
        subscribe_builder: SubscriptionBuilder,
    ) -> Result<(), tungstenite::Error> {
        let mut subscribe_message = json!({
            "action": "subscribe",
            "channels": []
        });

        fn append_channel(
            subscribe_message: &mut serde_json::Value,
            subscribe_builder: &SubscriptionBuilder,
            name: &str,
        ) {
            subscribe_message
                .as_object_mut()
                .unwrap()
                .get_mut("channels")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "name": name,
                    "markets": [ subscribe_builder.market ],
                }));
        }

        if subscribe_builder.trades {
            append_channel(&mut subscribe_message, &subscribe_builder, "trades");
        }
        if subscribe_builder.account {
            append_channel(&mut subscribe_message, &subscribe_builder, "account");
        }
        if subscribe_builder.book {
            append_channel(&mut subscribe_message, &subscribe_builder, "book");
        }
        if subscribe_builder.ticker {
            append_channel(&mut subscribe_message, &subscribe_builder, "ticker");
        }
        if subscribe_builder.candles {
            subscribe_message
                .as_object_mut()
                .unwrap()
                .get_mut("channels")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "name": "candles",
                    "interval": [ subscribe_builder.candles_interval ],
                    "markets": [ subscribe_builder.market ],
                }));
        }

        self.stream
            .send(tungstenite::Message::Text(
                subscribe_message.to_string().into(),
            ))
            .await
    }

    pub async fn pong(&mut self, bytes: Bytes) -> Result<(), tungstenite::Error> {
        self.stream.send(tungstenite::Message::Pong(bytes)).await
    }

    pub async fn get_markets(&mut self) -> Result<(), tungstenite::Error> {
        let markets_message = json!({
            "action": "getMarkets",
        });
        self.stream
            .send(tungstenite::Message::Text(
                markets_message.to_string().into(),
            ))
            .await
    }

    pub async fn get_balances(&mut self) -> Result<(), tungstenite::Error> {
        // this will return ALL the non-zero balances
        let get_balances = json!({
            "action": "privateGetBalance",
        });
        self.stream
            .send(tungstenite::Message::Text(get_balances.to_string().into()))
            .await
    }

    pub async fn place_buy_limit_order(
        &mut self,
        market: &str,
        quantity: FloatWrapper,
        price: FloatWrapper,
    ) -> Result<(), tungstenite::Error> {
        let order_message = json!({
            "action": "placeOrder",
            "market": market,
            "side": "buy",
            "orderType": "limit",
            "amount": quantity.to_string(),
            "price": price.to_string(),
        });

        self.stream
            .send(tungstenite::Message::Text(order_message.to_string().into()))
            .await
    }

    pub async fn place_sell_limit_order(
        &mut self,
        market: &str,
        quantity: FloatWrapper,
        price: FloatWrapper,
    ) -> Result<(), tungstenite::Error> {
        let order_message = json!({
            "action": "placeOrder",
            "market": market,
            "side": "sell",
            "orderType": "limit",
            "amount": quantity.to_string(),
            "price": price.to_string(),
        });

        self.stream
            .send(tungstenite::Message::Text(order_message.to_string().into()))
            .await
    }

    pub async fn place_buy_market_order(
        &mut self,
        market: &str,
        quantity: FloatWrapper,
    ) -> Result<(), tungstenite::Error> {
        let order_message = json!({
            "action": "placeOrder",
            "market": market,
            "side": "buy",
            "orderType": "market",
            "amount": quantity.to_string(),
        });

        self.stream
            .send(tungstenite::Message::Text(order_message.to_string().into()))
            .await
    }

    pub async fn place_sell_market_order(
        &mut self,
        market: &str,
        quantity: FloatWrapper,
    ) -> Result<(), tungstenite::Error> {
        let order_message = json!({
            "action": "placeOrder",
            "market": market,
            "side": "sell",
            "orderType": "market",
            "amount": quantity.to_string(),
        });

        self.stream
            .send(tungstenite::Message::Text(order_message.to_string().into()))
            .await
    }

    pub async fn cancel_order(&mut self, order_id: &str) -> Result<(), tungstenite::Error> {
        let cancel_message = json!({
            "action": "cancelOrder",
            "orderId": order_id,
        });

        self.stream
            .send(tungstenite::Message::Text(
                cancel_message.to_string().into(),
            ))
            .await
    }

    pub async fn cancel_all(&mut self) -> Result<(), tungstenite::Error> {
        let cancel_all_message = json!({
            "action": "cancelOrders",
        });
        self.stream
            .send(tungstenite::Message::Text(
                cancel_all_message.to_string().into(),
            ))
            .await
    }

    pub async fn cancel_all_within_market(
        &mut self,
        market: &str,
    ) -> Result<(), tungstenite::Error> {
        let cancel_all_message = json!({
            "action": "cancelOrders",
            "market": market,
        });
        self.stream
            .send(tungstenite::Message::Text(
                cancel_all_message.to_string().into(),
            ))
            .await
    }
}
