use crate::rug_float_serde::FloatWrapper;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use serde::de::{SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::{from_value, json};
use std::fmt::{Debug, Formatter};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

#[derive(Clone, Serialize, Default)]
pub struct PriceLevel {
    pub price: FloatWrapper,
    pub quantity: FloatWrapper,
}

impl Debug for PriceLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?} @ {:?}]", self.quantity, self.price)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Book {
    pub nonce: i32,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

impl<'de> Deserialize<'de> for PriceLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PriceLevelVisitor;

        impl<'de> Visitor<'de> for PriceLevelVisitor {
            type Value = PriceLevel;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON array representing price and quantity")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let price_str = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let quantity_str = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let price = from_value::<FloatWrapper>(price_str).map_err(de::Error::custom)?;
                let quantity =
                    from_value::<FloatWrapper>(quantity_str).map_err(de::Error::custom)?;

                Ok(PriceLevel { price, quantity })
            }
        }
        deserializer.deserialize_seq(PriceLevelVisitor)
    }
}

pub async fn get_book(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    market: &str,
) -> Result<(), tungstenite::Error> {
    let markets_message = json!({
        "action": "getBook",
        "market": market,
    });
    write.send(Message::Text(markets_message.to_string().into())).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deserialize_price_level() {
        let json_data = r#"
            ["100.50", "2.0"]
        "#;

        // Deserialize the JSON into a PriceLevel
        let price_level: PriceLevel = serde_json::from_str(json_data).unwrap();

        println!("{:?}", price_level);
    }
}
