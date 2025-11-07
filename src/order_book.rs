use crate::error::AggregatorError;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: BTreeMap<OrderedFloat<f64>, f64>,
    pub asks: BTreeMap<OrderedFloat<f64>, f64>,
}

impl OrderBook {
    // Create a new, empty OrderBook
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    // Add a bid level to the order book
    pub fn add_bid(&mut self, price: f64, quantity: f64) {
        *self.bids.entry(OrderedFloat(price)).or_insert(0.0) += quantity;
    }

    // Add an ask level to the order book
    pub fn add_ask(&mut self, price: f64, quantity: f64) {
        *self.asks.entry(OrderedFloat(price)).or_insert(0.0) += quantity;
    }

    pub fn merge(&mut self, other: &OrderBook) {
        for (price, quantity) in &other.bids {
            *self.bids.entry(*price).or_insert(0.0) += quantity;
        }
        for (price, quantity) in &other.asks {
            *self.asks.entry(*price).or_insert(0.0) += quantity;
        }
    }

    pub fn calculate_best_buy_offer(&self, quantity: f64) -> Result<f64, AggregatorError> {
        let mut remaining = quantity;
        let mut total_cost = 0.0;

        // Iterate through asks lowest first
        for (price, &qty_available) in &self.asks {
            if remaining <= 0.0 {
                break;
            }

            let qty_to_buy = remaining.min(qty_available);
            total_cost += qty_to_buy * price.0;
            remaining -= qty_to_buy;
        }

        if remaining > 0.0 {
            return Err(AggregatorError::InsufficientLiquidity(
                "Insufficient liquidity to complete order".to_string(),
            ));
        }
        total_cost = (total_cost * 100.0).round() / 100.0;
        Ok(total_cost)
    }

    pub fn calculate_best_sell_offer(&self, quantity: f64) -> Result<f64, AggregatorError> {
        let mut remaining = quantity;
        let mut total_cost = 0.0;

        // Iterate through bids highest first
        for (price, &qty_available) in self.bids.iter().rev() {
            if remaining <= 0.0 {
                break;
            }

            let qty_to_sell = remaining.min(qty_available);
            total_cost += qty_to_sell * price.0;
            remaining -= qty_to_sell;
        }

        if remaining > 0.0 {
            return Err(AggregatorError::InsufficientLiquidity(
                "Insufficient liquidity to complete order".to_string(),
            ));
        }
        total_cost = (total_cost * 100.0).round() / 100.0;
        Ok(total_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_best_buy_offer() {
        let mut order_book = OrderBook::new();
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
        assert_eq!(res, 10312.38);
    }

    #[test]
    fn test_best_sell_offer() {
        let mut order_book = OrderBook::new();
        order_book.add_bid(103120.00, 0.5);
        order_book.add_bid(103119.50, 0.3);
        order_book.add_bid(103118.00, 0.2);

        let res = order_book.calculate_best_sell_offer(0.4).unwrap();
        // Max bid price is 103120.00/BTC with quantity of 0.5 BTC
        // We can buy 0.4 BTC at total cost = (0.4 * 103120.00) =  41248.00
        assert_eq!(res, 41248.00);
    }

    #[test]
    fn test_insufficient_liquidity_buy() {
        let mut order_book = OrderBook::new();
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
        let mut order_book = OrderBook::new();
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
