mod aggregator;
mod data_providers;
mod order_book;
mod error;
use std::sync::Arc;

use clap::Parser;

use crate::data_providers::gemini::GeminiExchange;
use crate::{aggregator::OrderBookAggregator, data_providers::coinbase::CoinbaseExchange, error::AggregatorError};
use crate::data_providers::DataProvider;


#[derive(Parser, Debug)]
#[command(name = "order-book-aggregator")]
#[command(about = "BTC-USD Order Book Aggregator", long_about = None)]
struct Args {
    #[arg(long, default_value = "10.0")]
    qty: f64,
}

#[tokio::main]
async fn main()-> Result<(), AggregatorError>{
    let args = Args::parse();
    let quantity = args.qty;

    let data_providers = vec![
        Arc::new(CoinbaseExchange::new()) as Arc<dyn DataProvider>,
        Arc::new(GeminiExchange::new()) as Arc<dyn DataProvider>,
    ];
    
    let aggregator = OrderBookAggregator::new(data_providers, "BTC-USD");
    let aggregated_book = aggregator.fetch_and_aggregate_data().await?;

    let best_buy_price = aggregated_book.calculate_best_buy_offer(quantity)?;
    println!("To buy  {} BTC : ${}", quantity, best_buy_price);

    let best_sell_price = aggregated_book.calculate_best_sell_offer(quantity)?;
    println!("To sell {} BTC : ${}", quantity, best_sell_price);


    
    Ok(())
}
