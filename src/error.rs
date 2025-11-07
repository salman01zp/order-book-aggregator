use thiserror::Error;

#[derive(Debug, Error)]
pub enum AggregatorError {
    /// Insufficient liquidity error
    #[error("{0}")]
    InsufficientLiquidity(String),
    /// Aggregation Failed
    #[error("Failed to aggregate order books")]
    AggregationFailed,
    /// Rate limiter error
    #[error("{0}")]
    RateLimitExceeded(String),
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
    #[error("{0}")]
    ExchangeError(String),
    /// Environment variable error
    #[error(transparent)]
    DotenvyError(#[from] dotenvy::Error),
}
