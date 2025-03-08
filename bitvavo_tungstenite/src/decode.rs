use crate::candle::{Candle, CandleEvent};
use crate::event::{
    BitvavoEvent, BookResponse, GetBalancesResponse, SubscriptionResponse, Ticker, Ticker24h,
    TickerBookResponse,
};
use crate::market::MarketsResponse;
use crate::price_level::Book;
use std::collections::HashMap;

use crate::trade::Trade;
use serde_json::{from_value, Error};
use std::fmt::{Debug, Formatter};

pub enum DecodeError {
    NonDecodeableMessage(String),
    NonParseableMessage(serde_json::Error),
    UnknownEvent(String),
    UnknownActionType(String),
}

impl Debug for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::NonDecodeableMessage(s) => write!(f, "NonDecodeableMessage: {}", s),
            DecodeError::NonParseableMessage(e) => write!(f, "NonParseableMessage({:?})", e),
            DecodeError::UnknownEvent(e) => write!(f, "UnknownEvent({:?})", e),
            DecodeError::UnknownActionType(e) => write!(f, "UnknownActionType({:?})", e),
        }
    }
}

impl From<serde_json::Error> for DecodeError {
    fn from(value: Error) -> Self {
        DecodeError::NonParseableMessage(value)
    }
}

pub fn decode_event(message: &str) -> Result<BitvavoEvent, DecodeError> {
    let message_str = message.to_string();

    if !message_str.starts_with("{") {
        log::error!("message is weird: {}", message_str);
        return Err(DecodeError::NonDecodeableMessage(message_str.to_string()));
    }

    let value: serde_json::Value = serde_json::from_str(message_str.as_str())?;
    let maybe_event_type = value.get("event").and_then(|v| v.as_str());

    // events
    if let Some(event_type) = maybe_event_type {
        return match event_type {
            "subscribed" => Ok(BitvavoEvent::Subscribed),

            "book" => {
                let book = from_value::<Book>(value)?;
                Ok(BitvavoEvent::from_book(book))
            }

            "candle" => {
                let candle_response = from_value::<CandleEvent>(value)?;
                let candle = Candle::from_serde_array(candle_response.candle.first().unwrap());
                Ok(BitvavoEvent::from_candle(candle))
            }

            "trade" => {
                let trade = from_value::<Trade>(value)?;
                Ok(BitvavoEvent::from_trade(trade))
            }

            "ticker" => {
                let ticker = from_value::<Ticker>(value);
                match ticker {
                    Ok(ticker) => Ok(BitvavoEvent::from_ticker(ticker)),
                    Err(e) => {
                        log::error!("error: {:?}, payload = {}", &e, &message_str);
                        Err(DecodeError::NonParseableMessage(e))
                    }
                }
            }

            "ticker24h" => {
                let ticker = from_value::<Ticker24h>(value);
                match ticker {
                    Ok(ticker) => Ok(BitvavoEvent::from_ticker24h(ticker)),
                    Err(e) => {
                        log::error!("error: {:?}, payload = {}", &e, &message_str);
                        Err(DecodeError::UnknownEvent(e.to_string()))
                    }
                }
            }

            event => {
                log::info!("Unknown event type: {}", event_type);
                Err(DecodeError::UnknownEvent(event.to_string()))
            }
        };
    }

    let maybe_action_type = value.get("action").unwrap().as_str();

    // actions
    if let Some(action_type) = maybe_action_type {
        return match action_type {
            "getMarkets" => Ok(BitvavoEvent::Markets(
                from_value::<MarketsResponse>(value)?.response,
            )),

            "getTickerBook" => Ok(BitvavoEvent::TickerBook(from_value::<TickerBookResponse>(
                value,
            )?)),

            "privateGetBalance" => {
                let response = from_value::<GetBalancesResponse>(value)?;
                let balances = response
                    .response
                    .into_iter()
                    .map(|balance| (balance.symbol.clone(), balance))
                    .collect::<HashMap<_, _>>();
                Ok(BitvavoEvent::Balances(balances))
            }

            "getBook" => {
                let book_response = from_value::<BookResponse>(value)?;
                Ok(BitvavoEvent::Book(book_response.response))
            }

            "subscribe" => {
                let s_response = from_value::<SubscriptionResponse>(value)?;
                if let Some(e) = s_response.error {
                    panic!("subscription confirmation failed: {:?}", e);
                }
                Ok(BitvavoEvent::Subscribed)
            }

            action_type => {
                log::debug!("Unknown action type: {}", action_type);
                Err(DecodeError::UnknownActionType(action_type.to_string()))
            }
        };
    }

    // wtf
    panic!("{:?}", &value);
}
