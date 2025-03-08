use crate::rug_float_serde::FloatWrapper;
use crate::side::Side;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Trade {
    timestamp: u64,
    id: String,
    amount: FloatWrapper,
    price: FloatWrapper,
    side: Side,
}

impl Display for Trade {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trade [timestamp: {}, id: {}, amount: {}, price: {}, side: {}]",
            self.timestamp, self.id, self.amount, self.price, self.side
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_trade() {
        let s = serde_json::json!({
            "timestamp": 123123123,
            "id": "123",
            "amount": "123",
            "price": "123",
            "side": "sell",
        });

        serde_json::from_value::<Trade>(s).unwrap();
    }
}
