use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Buy => "buy",
                Side::Sell => "sell",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialize_side() {
        let buy = Side::Buy;
        let serialized = serde_json::to_string(&buy);
        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().as_str(), "\"buy\"");

        let buy = Side::Sell;
        let serialized = serde_json::to_string(&buy);
        assert!(serialized.is_ok());
        assert_eq!(serialized.unwrap().as_str(), "\"sell\"");
    }

    use super::*;
    #[test]
    fn deserialize_side_buy() {
        let buy = Side::Buy;
        let serialized = serde_json::to_string(&buy).unwrap();
        let deserialized: Side = serde_json::from_str(&serialized).unwrap();
        assert_eq!(buy, deserialized);
    }

    #[test]
    fn deserialize_side_sell() {
        let sell = Side::Sell;
        let serialized = serde_json::to_string(&sell).unwrap();
        let deserialized: Side = serde_json::from_str(&serialized).unwrap();
        assert_eq!(sell, deserialized);
    }
}
