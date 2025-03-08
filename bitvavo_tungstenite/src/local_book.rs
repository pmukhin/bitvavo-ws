use crate::get_book::{Book, PriceLevel};
use crate::rug_float_serde::FloatWrapper;

pub struct LocalBook {
    price_level_default: PriceLevel,
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

impl LocalBook {
    pub fn new() -> LocalBook {
        LocalBook {
            price_level_default: PriceLevel::default(),
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn top_bid_or_default(&self) -> &PriceLevel {
        self.bids.first().unwrap_or(&self.price_level_default)
    }

    pub fn top_ask_or_default(&self) -> &PriceLevel {
        self.asks.first().unwrap_or(&self.price_level_default)
    }

    pub fn real_spread_or_default(&self) -> FloatWrapper {
        let best_bid = self.bids.first();
        let best_ask = self.asks.first();
        if let Some(bid) = best_bid && let Some(ask) = best_ask {
            let market = (bid.price.float.clone() + ask.price.float.clone()) / 2;
            let spread = ask.price.float.clone() - bid.price.float.clone();
            return FloatWrapper::from(spread / market);
        }
        FloatWrapper::default()
    }

    pub fn digest(&mut self, book: Book) {
        let bids = book.bids.into_iter().skip_while(|pl| pl.quantity.float == 0);
        let asks = book.asks.into_iter().skip_while(|pl| pl.quantity.float == 0);
        self.bids = bids.collect();
        self.asks = asks.collect();
    }
}
