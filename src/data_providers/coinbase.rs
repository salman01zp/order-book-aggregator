use std::sync::Arc;

use crate::{
    data_providers::DataProvider, error::AggregatorError, order_book::OrderBook,
    rate_limiter::RateLimiter, types::Product,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
// Coinbase API response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoinbaseBookResponse {
    bids: Vec<(String, String, u32)>,
    asks: Vec<(String, String, u32)>,
}

// Coinbase Exchange Data Provider
pub struct CoinbaseExchange {
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl CoinbaseExchange {
    pub fn new() -> Self {
        CoinbaseExchange {
            client: reqwest::Client::new(),
            rate_limiter: Arc::new(Mutex::new(
                RateLimiter::new(1, 2), // 1 requests per 2 seconds.
            )),
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
    async fn fetch_order_book(&self, product_id: Product) -> Result<OrderBook, AggregatorError> {
        let base_url = "https://api.exchange.coinbase.com";
        let url = format!(
            "{}/products/{}/book?level=2",
            base_url,
            product_id.to_coinbase_symbol()
        );
        // Todo: Implement retry request client with backoff and retry policies to handle rate limits other errors.
        self.rate_limiter
            .lock()
            .await
            .check_if_rate_limited()
            .await?;
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
        let provider = CoinbaseExchange::new();
        let book = provider.fetch_order_book(Product::BTCUSD).await.unwrap();
        assert!(!book.is_empty());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let provider = CoinbaseExchange::new();
        // first request should pass
        assert!(provider.fetch_order_book(Product::BTCUSD).await.is_ok());
        // secong request should be rate limited
        let res = provider.fetch_order_book(Product::BTCUSD).await;
        assert!(matches!(res, Err(AggregatorError::RateLimitExceeded(_))));
    }
}
