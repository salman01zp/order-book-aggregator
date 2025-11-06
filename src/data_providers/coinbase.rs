use crate::{
    data_providers::{DataProvider, DataProviderError},
    types::{OrderBook, PriceLevel},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoinbaseBookResponse {
    bids: Vec<(String, String, u32)>,
    asks: Vec<(String, String, u32)>,
}

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

#[async_trait]
impl DataProvider for CoinbaseExchange {
    fn name(&self) -> &str {
        "Coinbase"
    }

    async fn fetch_order_book(
        &self,
        product_id: &str,
    ) -> Result<crate::types::OrderBook, crate::data_providers::DataProviderError> {
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
            return Err(DataProviderError::ExchangeError(
                "Failed to fetch order book from Coinbase ",
            ));
        }
        let book: CoinbaseBookResponse = response.json().await?;

        // Parse bids
        let bids: Vec<PriceLevel> = book
            .bids
            .iter()
            .filter_map(|level| {
                let price = level.0.parse::<f64>().ok()?;
                let quantity = level.1.parse::<f64>().ok()?;
                Some(PriceLevel { price, quantity })
            })
            .collect();

        // Parse asks
        let asks: Vec<PriceLevel> = book
            .asks
            .iter()
            .filter_map(|level| {
                let price = level.0.parse::<f64>().ok()?;
                let quantity = level.1.parse::<f64>().ok()?;
                Some(PriceLevel { price, quantity })
            })
            .collect();

        Ok(OrderBook { bids, asks })
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
