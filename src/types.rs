// Supported products for aggregation
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub enum Product {
    BTCUSD,
}

impl Product {
    pub fn to_coinbase_symbol(&self) -> &str {
        match self {
            Product::BTCUSD => "BTC-USD",
        }
    }

    pub fn to_gemini_symbol(&self) -> &str {
        match self {
            Product::BTCUSD => "BTCUSD",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Exchange {
    Coinbase,
    Gemini,
    AggregatedExchange,
}

impl Exchange {
    pub fn to_string(&self) -> String {
        match self {
            Exchange::Coinbase => "coinbase".to_string(),
            Exchange::Gemini => "gemini".to_string(),
            Exchange::AggregatedExchange => "agg".to_string(),
        }
    }
}
