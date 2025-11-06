mod coinbase;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataProviderError {
    /// JSON Error occurred.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Failed to convert string to float
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    /// Exchange error.
    #[error("{}", _0)]
    ExchangeError(&'static str),
}

#[async_trait]
pub trait DataProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch_order_book(
        &self,
        product_id: &str,
    ) -> Result<crate::types::OrderBook, DataProviderError>;
}
