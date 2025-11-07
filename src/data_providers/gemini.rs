use crate::{
    data_providers::DataProvider, error::AggregatorError, order_book::OrderBook,
    rate_limiter::RateLimiter, types::Product,
};
use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
// Gemini API response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPricelevel {
    price: String,
    amount: String,
    timestamp: String,
}

// Gemini API response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiBookResponse {
    bids: Vec<GeminiPricelevel>,
    asks: Vec<GeminiPricelevel>,
}

// Gemini Exchange Data Provider
pub struct GeminiExchange {
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    base_url: Url,
}

impl GeminiExchange {
    pub fn new() -> Self {
        let url = dotenvy::var("GEMINI_API_BASE_URL").expect("Failed to get gemini base url from env");
        let base_url = Url::parse(&url).expect("Invalid Gemini API base URL");
        GeminiExchange {
            client: reqwest::Client::new(),
            rate_limiter: Arc::new(Mutex::new(
                RateLimiter::new(1, 2), // 1 requests per 2 seconds.
            )),
            base_url,
        }
    }
}

// Implement DataProvider trait for GeminiExchange
#[async_trait]
impl DataProvider for GeminiExchange {
    fn name(&self) -> &str {
        "Gemini"
    }

    // Fetch order book data from Gemini API
    async fn fetch_order_book(&self, product_id: Product) -> Result<OrderBook, AggregatorError> {
        let url = format!("{}v1/book/{}", self.base_url, product_id.to_gemini_symbol());
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
            let err: String = response.text().await?;
            return Err(AggregatorError::ExchangeError(format!(
                "Failed to fetch order book from Gemini :  {}",
                err
            )));
        }
        let book: GeminiBookResponse = response.json().await?;
        let mut order_book = OrderBook::new();
        // Add bids to order book
        for level in &book.bids {
            if let (Ok(price), Ok(quantity)) =
                (level.price.parse::<f64>(), level.amount.parse::<f64>())
            {
                order_book.add_bid(price, quantity);
            }
        }
        // Add asks to order book
        for level in &book.asks {
            if let (Ok(price), Ok(quantity)) =
                (level.price.parse::<f64>(), level.amount.parse::<f64>())
            {
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
    async fn test_fetch_gemini_order_book() {
        let exchange = GeminiExchange::new();
        let order_book = exchange.fetch_order_book(Product::BTCUSD).await.unwrap();
        assert!(!order_book.is_empty());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let provider = GeminiExchange::new();
        // first request should pass
        assert!(provider.fetch_order_book(Product::BTCUSD).await.is_ok());
        // secong request should be rate limited
        let res = provider.fetch_order_book(Product::BTCUSD).await;
        assert!(matches!(res, Err(AggregatorError::RateLimitExceeded(_))));
    }
}
