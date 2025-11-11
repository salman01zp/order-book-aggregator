use crate::{
    data_providers::DataProvider,
    error::AggregatorError,
    order_book::OrderBook,
    types::{Exchange, Product},
};
use std::sync::Arc;

pub struct OrderBookAggregator {
    // Data providers to fetch order book data from
    data_providers: Vec<Arc<dyn crate::data_providers::DataProvider>>,
    // OrderBook aggregation for this product ID
    product_id: Product,
}

impl OrderBookAggregator {
    // Create a new OrderBookAggregator
    pub fn new(data_providers: Vec<Arc<dyn DataProvider>>, product_id: Product) -> Self {
        OrderBookAggregator {
            data_providers,
            product_id,
        }
    }

    // Fetch and aggregate order book data from all data providers
    pub async fn fetch_and_aggregate_data(&self) -> Result<OrderBook, AggregatorError> {
        let mut handles = Vec::new();
        for provider in &self.data_providers {
            let provider = Arc::clone(provider);
            let product_id = self.product_id.clone();
            let handle = tokio::spawn(async move {
                let name = provider.name().to_string();
                match provider.fetch_order_book(product_id).await {
                    Ok(book) => Ok(book),
                    Err(e) => Err((name, e)),
                }
            });
            handles.push(handle);
        }
        let mut aggregated_book = OrderBook::new(Exchange::AggregatedExchange);
        // Every sucessfull data fetch task will be marked and counted as handled.
        let mut marked_as_handled = 0;

        for handle in handles {
            match handle.await {
                Ok(Ok(book)) => {
                    marked_as_handled += 1;
                    if aggregated_book.is_empty() {
                        aggregated_book = book;
                        continue;
                    }
                    aggregated_book.merge(&book);
                }
                Ok(Err((provider_name, error))) => {
                    println!(
                        "Warning: Failed to fetch data from {}: {}",
                        provider_name, error
                    );
                }
                Err(join_error) => {
                    println!("Warning: Task join error: {}", join_error);
                }
            };
        }

        // Only fail if ALL providers failed.
        if marked_as_handled == 0 {
            return Err(AggregatorError::AggregationFailed);
        }

        Ok(aggregated_book)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_providers::{coinbase::CoinbaseExchange, gemini::GeminiExchange};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_aggregator() {
        let coinbase = Arc::new(CoinbaseExchange::new());
        let aggregator = OrderBookAggregator::new(vec![coinbase], Product::BTCUSD);
        let aggregated_book = aggregator.fetch_and_aggregate_data().await.unwrap();
        assert!(!aggregated_book.is_empty());
    }

    #[tokio::test]
    async fn test_aggregator_with_multi_providers() {
        let provider1 = Arc::new(CoinbaseExchange::new());
        let provider2 = Arc::new(GeminiExchange::new());
        // Here we can add more mock providers for testing
        let aggregator = OrderBookAggregator::new(vec![provider1, provider2], Product::BTCUSD);
        let aggregated_book = aggregator.fetch_and_aggregate_data().await.unwrap();
        assert!(!aggregated_book.is_empty());
    }
}
