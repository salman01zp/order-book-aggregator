use std::collections::BTreeMap;

use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: BTreeMap<OrderedFloat<f64>, f64>,
    pub asks: BTreeMap<OrderedFloat<f64>, f64>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_bid(&mut self, price: f64, quantity: f64) {
        *self.bids.entry(OrderedFloat(price)).or_insert(0.0) += quantity;
    }

    pub fn add_ask(&mut self, price: f64, quantity: f64) {
        *self.asks.entry(OrderedFloat(price)).or_insert(0.0) += quantity;
    }
}
