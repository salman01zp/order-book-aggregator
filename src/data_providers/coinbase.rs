use crate::{data_providers::DataProvider, error::AggregatorError, order_book::OrderBook};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Coinbase API response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoinbaseBookResponse {
    bids: Vec<(String, String, u32)>,
    asks: Vec<(String, String, u32)>,
}

// Coinbase Exchange Data Provider
pub struct CoinbaseExchange {
    client: reqwest::Client,
}

impl CoinbaseExchange {
    pub fn new() -> Self {
        CoinbaseExchange {
            client: reqwest::Client::new(),
        }
    }
}

// Implement DataProvider trait for CoinbaseExchange
#[async_trait]
impl DataProvider for CoinbaseExchange {
    fn name(&self) -> &str {
        "Coinbase"
    }

    // Fetch order book data from Coinbase API
    async fn fetch_order_book(&self, product_id: &str) -> Result<OrderBook, AggregatorError> {
        let base_url = "https://api.exchange.coinbase.com";
        let url = format!("{}/products/{}/book?level=2", base_url, product_id);
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "order-book-aggregator/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            let res = response.text().await?;
            println!("Coinbase API error response: {}", res);
            return Err(AggregatorError::ExchangeError(
                "Failed to fetch order book from Coinbase ",
            ));
        }
        let book: CoinbaseBookResponse = response.json().await?;
        let mut order_book = OrderBook::new();
        // Add bids to order book
        for level in &book.bids {
            if let (Ok(price), Ok(quantity)) = (level.0.parse::<f64>(), level.1.parse::<f64>()) {
                order_book.add_bid(price, quantity);
            }
        }
        // Add asks to order book
        for level in &book.asks {
            if let (Ok(price), Ok(quantity)) = (level.0.parse::<f64>(), level.1.parse::<f64>()) {
                order_book.add_ask(price, quantity);
            }
        }

        Ok(order_book)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_coinbase_order_book() {
        let exchange = CoinbaseExchange::new();
        let result = exchange.fetch_order_book("BTC-USD").await;
        let book = result.unwrap();
        assert!(!book.bids.is_empty());
        assert!(!book.asks.is_empty());
    }
}
