use crate::candle::Candle;
use crate::get_book::Book;
use crate::get_markets::Market;
use crate::rug_float_serde::FloatWrapper;
use crate::sig::create_signature;
use crate::trade::Trade;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub enum BitvavoEvent {
    Subscribed,
    Book(Book),
    Candle(Candle),
    Trade(Trade),
    Markets(Vec<Market>),
    TickerBook(TickerBookResponse),
    Ticker24h(Ticker24h),
    Ticker(Ticker),
}

impl BitvavoEvent {
    pub fn from_candle(candle: Candle) -> Self {
        BitvavoEvent::Candle(candle)
    }

    pub fn from_book(book: Book) -> Self {
        BitvavoEvent::Book(book)
    }

    pub fn from_trade(trade: Trade) -> Self {
        BitvavoEvent::Trade(trade)
    }

    pub fn from_ticker24h(ticker24h: Ticker24h) -> Self {
        BitvavoEvent::Ticker24h(ticker24h)
    }

    pub fn from_ticker(ticker: Ticker) -> Self {
        BitvavoEvent::Ticker(ticker)
    }
}

// TimeResponse and Time
#[derive(Serialize, Deserialize)]
pub struct TimeResponse {
    action: String,
    response: Time,
}

#[derive(Serialize, Deserialize)]
pub struct Time {
    time: u32,
}

// AssetsResponse and Assets
#[derive(Serialize, Deserialize)]
pub struct AssetsResponse {
    action: String,
    response: Vec<Assets>,
}

#[derive(Serialize, Deserialize)]
pub struct Assets {
    symbol: String,
    name: String,
    decimals: u32,
    deposit_fee: String,
    deposit_confirmations: u32,
    deposit_status: String,
    withdrawal_fee: String,
    withdrawal_min_amount: String,
    withdrawal_status: String,
    networks: Vec<String>,
    message: String,
}

// BookResponse and Book
#[derive(Serialize, Deserialize)]
pub struct BookResponse {
    pub response: Book,
}

// PublicTradesResponse and PublicTrades
#[derive(Serialize, Deserialize)]
pub struct PublicTradesResponswe {
    action: String,
    response: Vec<Trade>,
}

// Ticker24hResponse and Ticker24h
#[derive(Serialize, Deserialize)]
pub struct Ticker24hResponse {
    action: String,
    response: Vec<Ticker24h>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker {
    market: String,
    #[serde(rename = "bestBid")]
    best_bid: Option<FloatWrapper>,
    #[serde(rename = "bestBidSize")]
    best_bid_size: Option<FloatWrapper>,
    #[serde(rename = "bestAsk")]
    best_ask: Option<FloatWrapper>,
    #[serde(rename = "bestAskSize")]
    best_ask_size: Option<FloatWrapper>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker24h {
    market: String,
    open: String,
    high: String,
    low: String,
    last: String,
    volume: String,
    volume_quote: String,
    bid: String,
    ask: String,
    timestamp: u64,
    bid_size: String,
    ask_size: String,
}

#[derive(Serialize, Deserialize)]
pub struct TickerPriceResponse {
    action: String,
    response: Vec<TickerPrice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickerPrice {
    market: String,
    price: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickerBookResponse {
    action: String,
    response: TickerBook,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionResponse {
    pub error: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TickerBook {
    market: String,
    bid: String,
    ask: String,
    bid_size: String,
    ask_size: String,
}

// PlaceOrderResponse, GetOrderResponse, UpdateOrderResponse, and CancelOrderResponse
#[derive(Serialize, Deserialize, Debug)]
pub struct PlaceOrderResponse {
    action: String,
    response: Order,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOrderResponse {
    action: String,
    response: Order,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateOrderResponse {
    action: String,
    response: Order,
}

#[derive(Serialize, Deserialize)]
pub struct CancelOrderResponse {
    action: String,
    response: CancelOrder,
}

// Order and CancelOrder
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    order_id: String,
    market: String,
    created: u32,
    updated: u32,
    status: String,
    side: String,
    order_type: String,
    amount: String,
    amount_remaining: String,
    price: String,
    amount_quote: String,
    amount_quote_remaining: String,
    on_hold: String,
    on_hold_currency: String,
    filled_amount: String,
    filled_amount_quote: String,
    fee_paid: String,
    fee_currency: String,
    fills: Vec<Fill>,
    self_trade_prevention: String,
    visible: bool,
    disable_market_protection: bool,
    time_in_force: String,
    post_only: bool,
    trigger_amount: String,
    trigger_price: String,
    trigger_type: String,
    trigger_reference: String,
}

#[derive(Serialize, Deserialize)]
pub struct CancelOrder {
    order_id: String,
}

// Fill
#[derive(Serialize, Deserialize, Debug)]
struct Fill {
    id: String,
    timestamp: u64,
    amount: String,
    price: String,
    taker: bool,
    fee: String,
    fee_currency: String,
    settled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub symbol: String,
    pub available: String,
    pub in_order: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBalancesResponse {
    pub response: Vec<Balance>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthRequest {
    action: String,
    key: String,
    signature: String,
    timestamp: u64,
    window: String,
}

impl AuthRequest {
    pub fn make(api_key: &str, api_secret: &str) -> Self {
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH).unwrap().as_millis();
        let timestamp_as_str = timestamp.to_string();

        AuthRequest {
            action: "authenticate".to_string(),
            key: api_key.to_string(),
            signature: create_signature(
                &timestamp_as_str,
                "GET",
                "/websocket",
                HashMap::new(),
                api_secret,
            ),
            timestamp: timestamp as u64,
            window: "1500".to_string(), // 1.5 seconds
        }
    }
}
