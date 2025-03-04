# Bitquery Kafka Solana Connector

This repository provides a Rust implementation for consuming Solana blockchain data from Bitquery's Kafka streaming service. It demonstrates how to connect to Bitquery's Kafka endpoints, parse Protobuf messages containing blockchain data, and process trade events.

## Project Structure

```
/bitquery-kafka-solana
├── solana/                  # Protobuf definition files
│   ├── bigtable_block.proto
│   ├── block_message.proto
│   ├── dex_block_message.proto
│   ├── parsed_idl_block_message.proto
│   └── token_block_message.proto
├── src/
│   ├── protos/
│   │   └── mod.rs          # Generated protobuf module
│   └── main.rs             # Main application code
├── build.rs                # Rust build script for protobuf compilation
└── Cargo.toml              # Rust dependencies and project config
```

## Features

- Connects to Bitquery's Kafka streaming service with SASL_SSL authentication
- Consumes Solana DEX trade events in real-time
- Parses Protobuf-encoded blockchain data
- Calculates token prices in USDC
- Implements bridging logic for tokens without direct USDC pairs
- Handles connection timeouts and error recovery

## Prerequisites

- Rust and Cargo installed
- SSL certificates for Kafka authentication:
    - `client.cer.pem`
    - `client.key.pem`
    - `server.cer.pem`
- Bitquery API credentials

## Setup

1. Clone the repository:
```bash
git clone https://github.com/yourusername/bitquery-kafka-solana.git
cd bitquery-kafka-solana
```

2. Place your SSL certificates in the project directory.

3. Update the Kafka configuration in `src/main.rs` with your Bitquery credentials:
```rust
.set("bootstrap.servers", "rpk0.bitquery.io:9093,rpk1.bitquery.io:9093,rpk2.bitquery.io:9093")
.set("sasl.username", "YOUR_USERNAME")
.set("sasl.password", "YOUR_PASSWORD")
.set("group.id", "YOUR_GROUP_ID")
```

4. Build the project:
```bash
cargo build --release
```

## Running the Connector

```bash
cargo run --release
```

By default, the connector subscribes to the `solana.dextrades.proto` Kafka topic. You can modify the topics array in `main.rs` to subscribe to additional Bitquery streams.

## Sample Output

The connector outputs token prices in real-time:

```
Token SOL price in USDC: 26.45
Updated last SOL price in USDC => Some(26.45)
Token RAY price in USDC (via SOL bridging): 0.32
```

## Customization

- Change the subscription topics in the `topics` array
- Modify the price calculation logic in `calculate_trade_price`
- Implement additional bridging logic for other tokens
- Add support for other Bitquery streaming events

## Bitquery Documentation

For more information about Bitquery's Kafka streaming service and available topics, visit:
- [Bitquery Documentation](https://docs.bitquery.io/)
- [Bitquery Streaming Service](https://streaming.bitquery.io/)

## License

[MIT License](LICENSE)

## Acknowledgements

This project uses the following Rust crates:
- `rdkafka` for Kafka connectivity
- `tokio` for async runtime
- `prost` for Protobuf message parsing
- `env_logger` for logging

## Support

For questions related to Bitquery services, please contact Bitquery support.
For issues with this connector, please open an issue on GitHub.
