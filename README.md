# Order Book Aggregator

Aggregates order book data from multiple exchanges.


### Prerequisites

Install and configure rustup:

```bash
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```

Configure the Rust toolchain to default to the latest stable version:

```bash
rustup default stable
rustup update
```

Great! Now your Rust environment is ready!

## Setup

```bash
git clone https://github.com/salman01zp/order-book-aggregator
cd order-book-aggregator
```

## Configure Environment Variables
```bash
cp .env.example .env
```

## Build
```bash
cargo build --release
```
## Run

Run with default quantity 10:
```bash
./target/release/order-book-aggregator 
```

Run with custom quantity:
```bash
./target/release/order-book-aggregator  --qty 5
```

## Testing

```bash
cargo test
```
