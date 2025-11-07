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
