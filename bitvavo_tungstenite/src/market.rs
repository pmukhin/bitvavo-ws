use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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
