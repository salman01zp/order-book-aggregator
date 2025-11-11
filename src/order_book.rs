use crate::{error::AggregatorError, types::Exchange};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Level {
    price: OrderedFloat<f64>,
    quantity: OrderedFloat<f64>,
    exchnage: Exchange,
}

#[derive(Debug, Clone)]
pub struct OrderDetails {
    price: f64,
    quantity: f64,
    exchange: String,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: BTreeMap<OrderedFloat<f64>, Level>,
    pub asks: BTreeMap<OrderedFloat<f64>, Level>,
    pub exchange: Exchange,
}

impl OrderBook {
    // Create a new, empty OrderBook
    pub fn new(exchange: Exchange) -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            exchange,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    // Add a bid level to the order book
    pub fn add_bid(&mut self, price: f64, quantity: f64) {
        let level = Level {
            price: OrderedFloat(price),
            exchnage: self.exchange,
            quantity: OrderedFloat(quantity),
        };

        match self.bids.get_mut(&OrderedFloat(price)) {
            Some(value) => {
                *value.quantity = *value.quantity + *level.quantity;
            }
            None => {
                self.bids.insert(OrderedFloat(price), level);
            }
        }
    }

    // Add an ask level to the order book
    pub fn add_ask(&mut self, price: f64, quantity: f64) {
        let level = Level {
            price: OrderedFloat(price),
            exchnage: self.exchange,
            quantity: OrderedFloat(quantity),
        };

        match self.asks.get_mut(&OrderedFloat(price)) {
            Some(value) => {
                *value.quantity = *value.quantity + *level.quantity;
            }
            None => {
                self.asks.insert(OrderedFloat(price), level);
            }
        }
    }

    pub fn merge(&mut self, other: &OrderBook) {
        for (price, level) in &other.bids {
            match self.bids.get_mut(price) {
                Some(value) => {
                    *value.quantity = *value.quantity + *level.quantity;
                }
                None => {
                    self.bids.insert(*price, level.clone());
                }
            }
        }
        for (price, level) in &other.asks {
            match self.asks.get_mut(price) {
                Some(value) => {
                    *value.quantity = *value.quantity + *level.quantity;
                }
                None => {
                    self.asks.insert(*price, level.clone());
                }
            }
        }
    }

    pub fn calculate_best_buy_offer(
        &self,
        quantity: f64,
    ) -> Result<Vec<OrderDetails>, AggregatorError> {
        let mut remaining = quantity;
        let mut total_cost = 0.0;
        let mut order_fullfilment = Vec::new();
        // Iterate through asks lowest first
        for (price, level) in &self.asks {
            if remaining <= 0.0 {
                break;
            }
            let qty_to_buy = remaining.min(level.quantity.0);

            total_cost += qty_to_buy * price.0;
            remaining -= qty_to_buy;
            let order_fullfilment_details = OrderDetails {
                price: price.0,
                quantity: qty_to_buy,
                exchange: level.exchnage.to_string(),
            };
            order_fullfilment.push(order_fullfilment_details)
        }

        if remaining > 0.0 {
            return Err(AggregatorError::InsufficientLiquidity(
                "Insufficient liquidity to complete order".to_string(),
            ));
        }
        total_cost = (total_cost * 100.0).round() / 100.0;
        Ok(order_fullfilment)
    }

    pub fn calculate_best_sell_offer(
        &self,
        quantity: f64,
    ) -> Result<Vec<OrderDetails>, AggregatorError> {
        let mut remaining = quantity;
        let mut total_cost = 0.0;
        let mut order_fullfilment = Vec::new();
        // Iterate through bids highest first
        for (price, level) in self.bids.iter().rev() {
            if remaining <= 0.0 {
                break;
            }

            let qty_to_sell = remaining.min(level.quantity.0);
            total_cost += qty_to_sell * price.0;
            remaining -= qty_to_sell;
            let order_fullfilment_details = OrderDetails {
                price: price.0,
                quantity: qty_to_sell,
                exchange: level.exchnage.to_string(),
            };
            order_fullfilment.push(order_fullfilment_details)
        }

        if remaining > 0.0 {
            return Err(AggregatorError::InsufficientLiquidity(
                "Insufficient liquidity to complete order".to_string(),
            ));
        }
        total_cost = (total_cost * 100.0).round() / 100.0;
        Ok(order_fullfilment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_best_buy_offer() {
        let mut order_book = OrderBook::new(Exchange::Coinbase);
        order_book.add_ask(103134.09, 0.50608469);
        order_book.add_ask(103134.02, 0.00803816);
        order_book.add_ask(103134.01, 0.09985862);
        order_book.add_ask(103133.06, 0.00969621);
        order_book.add_ask(103126.01, 0.1378704);
        order_book.add_ask(103123.79, 0.1425);

        let res = order_book.calculate_best_buy_offer(0.1).unwrap();
        // Min ask price is 103123.79/BTC with quantity of 0.1425 BTC
        // So total cost = 0.1 * 103123.79 = 10312.379
        // Rounded to 2 decimal places = 10312.38
    }

    #[test]
    fn test_best_sell_offer() {
        let mut order_book = OrderBook::new(Exchange::Gemini);
        order_book.add_bid(103120.00, 0.5);
        order_book.add_bid(103119.50, 0.3);
        order_book.add_bid(103118.00, 0.2);

        let res = order_book.calculate_best_sell_offer(0.4).unwrap();
        // Max bid price is 103120.00/BTC with quantity of 0.5 BTC
        // We can buy 0.4 BTC at total cost = (0.4 * 103120.00) =  41248.00
        // assert_eq!(res, 41248.00);
    }

    #[test]
    fn test_insufficient_liquidity_buy() {
        let mut order_book = OrderBook::new(Exchange::Coinbase);
        // We have liquidity 0.2 BTC
        order_book.add_ask(103118.00, 0.2);
        // We are trying to buy 0.5 BTC and this should fail with InsufficientLiquidity error
        let res = order_book.calculate_best_buy_offer(0.5);
        assert!(matches!(
            res,
            Err(AggregatorError::InsufficientLiquidity { .. })
        ));
    }

    #[test]
    fn test_insufficient_liquidity_sell() {
        let mut order_book = OrderBook::new(Exchange::Gemini);
        // We have liquidity 0.3 BTC
        order_book.add_bid(103118.00, 0.3);
        // We are trying to sell 0.6 BTC and this should fail with InsufficientLiquidity error
        let res = order_book.calculate_best_sell_offer(0.6);
        assert!(matches!(
            res,
            Err(AggregatorError::InsufficientLiquidity { .. })
        ));
    }
}
