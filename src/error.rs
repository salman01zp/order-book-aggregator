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
    #[error("{}", _0)]
    ExchangeError(&'static str),
}
