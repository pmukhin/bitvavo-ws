use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CandleEvent {
    pub market: String,
    pub candle: Vec<Vec<serde_json::Value>>,
    pub interval: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candle {
    pub timestamp: u64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

impl Display for Candle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Candle [timestamp: {}, open: {}, high: {}, low: {}, close: {}, volume: {}]",
            self.timestamp, self.open, self.high, self.low, self.close, self.volume
        )
    }
}

impl Candle {
    pub fn from_serde_array(array: &[serde_json::Value]) -> Candle {
        Candle {
            timestamp: array[0].as_u64().unwrap(),
            open: array[1].as_str().unwrap().to_string(),
            high: array[2].as_str().unwrap().to_string(),
            low: array[3].as_str().unwrap().to_string(),
            close: array[4].as_str().unwrap().to_string(),
            volume: array[5].as_str().unwrap().to_string(),
        }
    }
}
