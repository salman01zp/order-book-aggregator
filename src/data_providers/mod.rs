use crate::{error::AggregatorError, order_book::OrderBook};
use async_trait::async_trait;
pub mod coinbase;
pub mod gemini;

#[async_trait]
pub trait DataProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch_order_book(&self, product_id: &str) -> Result<OrderBook, AggregatorError>;
}
