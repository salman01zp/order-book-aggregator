use crate::data_providers::DataProviderError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AggregatorError {
    /// Insufficient liquidity error
    #[error(
        "Insufficient liquidity on {side}: requested {requested} BTC, only {available} BTC available"
    )]
    InsufficientLiquidity {
        side: String,
        requested: f64,
        available: f64,
    },
    /// Aggregation Failed
    #[error("Failed to aggregate order books")]
    AggregationFailed,
    /// DataProviderError error
    #[error(transparent)]
    DataProviderError(#[from] DataProviderError),
}
