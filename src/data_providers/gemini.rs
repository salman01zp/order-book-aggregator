use crate::{
    data_providers::{DataProvider, DataProviderError},
    order_book::OrderBook,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPricelevel {
    price: String,
    amount: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiBookResponse {
    bids: Vec<GeminiPricelevel>,
    asks: Vec<GeminiPricelevel>,
}

pub struct GeminiExchange {
    client: reqwest::Client,
}

impl GeminiExchange {
    pub fn new() -> Self {
        GeminiExchange {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DataProvider for GeminiExchange {
    fn name(&self) -> &str {
        "Gemini"
    }

    async fn fetch_order_book(
        &self,
        product_id: &str,
    ) -> Result<OrderBook, DataProviderError> {
        let base_url = "https://api.gemini.com";
        let url = format!("{}/v1/book/{}", base_url, product_id);
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "order-book-aggregator/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            let res = response.text().await?;
            println!("Gemini API error response: {}", res);
            return Err(DataProviderError::ExchangeError(
                "Failed to fetch order book from Gemini",
            ));
        }
        let book: GeminiBookResponse = response.json().await?;
        let mut order_book = OrderBook::new();
        // Add bids to order book
        for level in &book.bids {
            if let (Ok(price), Ok(quantity)) = (level.price.parse::<f64>(), level.amount.parse::<f64>()) {
                order_book.add_bid(price, quantity);
            }
        }
        // Add asks to order book
        for level in &book.asks {
            if let (Ok(price), Ok(quantity)) = (level.price.parse::<f64>(), level.amount.parse::<f64>()) {
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
        let order_book = exchange.fetch_order_book("BTCUSD").await.unwrap();
        assert!(!order_book.is_empty());
    }
}

    

