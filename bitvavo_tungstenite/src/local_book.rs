use crate::event::Ticker;
use crate::price_level::{Book, PriceLevel};
use crate::rug_float_serde::FloatWrapper;

#[derive(Debug, Default)]
pub struct LocalBook {
    price_level_default: PriceLevel,
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

impl LocalBook {
    pub fn top_bid_or_default(&self) -> &PriceLevel {
        self.bids.first().unwrap_or(&self.price_level_default)
    }

    pub fn top_ask_or_default(&self) -> &PriceLevel {
        self.asks.first().unwrap_or(&self.price_level_default)
    }

    pub fn real_spread_or_default(&self) -> FloatWrapper {
        let best_bid = self.bids.first();
        let best_ask = self.asks.first();
        if let Some(bid) = best_bid
            && let Some(ask) = best_ask
        {
            let market = (bid.price.float.clone() + ask.price.float.clone()) / 2;
            let spread = ask.price.float.clone() - bid.price.float.clone();
            return FloatWrapper::from(spread / market);
        }
        FloatWrapper::default()
    }

    // when ingesting a ticker only the top of the book is available
    pub fn ingest_ticker(&mut self, ticker: Ticker) {
        if let Some(best_bid) = ticker.best_bid {
            self.bids = Vec::new();
            self.bids.push(PriceLevel {
                price: best_bid,
                quantity: ticker.best_bid_size.unwrap(),
            })
        }
        if let Some(best_ask) = ticker.best_ask {
            self.asks = Vec::new();
            self.asks.push(PriceLevel {
                price: best_ask,
                quantity: ticker.best_ask_size.unwrap(),
            })
        }
    }

    pub fn ingest_book(&mut self, book: Book) {
        let bids = book
            .bids
            .into_iter()
            .skip_while(|pl| pl.quantity.float == 0);
        let asks = book
            .asks
            .into_iter()
            .skip_while(|pl| pl.quantity.float == 0);
        self.bids = bids.collect();
        self.asks = asks.collect();
    }
}
