mod protos;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::error::KafkaResult;
use tokio::time::{timeout, Duration};
use prost::Message as ProstMessage;
use crate::protos::dex_block_message::DexParsedBlockMessage;
use bs58;
use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AuthConfig {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Settings {
    auth: AuthConfig,
}

fn is_wsol(mint_address_bytes: &[u8]) -> bool {
    let mint_address_base58 = bs58::encode(mint_address_bytes).into_string();
    mint_address_base58 == "So11111111111111111111111111111111111111112"
}

#[tokio::main]
async fn main() -> KafkaResult<()> {
    env_logger::init();

    // Load config from config.toml
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();

    let settings: Settings = settings.try_deserialize().unwrap();

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "rpk0.bitquery.io:9092,rpk1.bitquery.io:9092,rpk2.bitquery.io:9092")
        .set("security.protocol", "SASL_PLAINTEXT")
        .set("ssl.endpoint.identification.algorithm", "none")
        .set("sasl.mechanisms", "SCRAM-SHA-512")
        .set("sasl.username", &settings.auth.username)
        .set("sasl.password", &settings.auth.password)
        .set("group.id", "solana_105-group-11")
        .set("fetch.message.max.bytes", "10485760")
        .create()?;

    let topics = vec!["solana.dextrades.proto"];
    consumer.subscribe(&topics)?;

    println!("Starting consumer, listening on topics: {:?}", topics);

    let mut last_sol_price_in_usdc: Option<f64> = None;
    let known_usdc_symbol = "USDC";

    loop {
        match timeout(Duration::from_secs(10), consumer.recv()).await {
            Ok(msg_result) => match msg_result {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        match DexParsedBlockMessage::decode(payload) {
                            Ok(parsed_block) => {
                                for dex_tx in &parsed_block.transactions {
                                    if let Some(status) = &dex_tx.status {
                                        if status.success {
                                            for trade in &dex_tx.trades {
                                                if let (Some(buy_side), Some(sell_side)) = (&trade.buy, &trade.sell) {
                                                    let buy_currency = match &buy_side.currency {
                                                        Some(cur) => cur,
                                                        None => continue,
                                                    };
                                                    let sell_currency = match &sell_side.currency {
                                                        Some(cur) => cur,
                                                        None => continue,
                                                    };

                                                    let buy_amount = buy_side.amount as f64
                                                        / 10f64.powi(buy_currency.decimals as i32);
                                                    let sell_amount = sell_side.amount as f64
                                                        / 10f64.powi(sell_currency.decimals as i32);

                                                    if buy_amount == 0.0 || sell_amount == 0.0 {
                                                        continue;
                                                    }

                                                    let buy_is_usdc = buy_currency.symbol == known_usdc_symbol;
                                                    let sell_is_usdc = sell_currency.symbol == known_usdc_symbol;

                                                    let buy_is_sol = is_wsol(&buy_currency.mint_address);
                                                    let sell_is_sol = is_wsol(&sell_currency.mint_address);

                                                    if buy_is_usdc || sell_is_usdc {
                                                        let token_price_in_usdc = if buy_is_usdc {
                                                            buy_amount / sell_amount
                                                        } else {
                                                            sell_amount / buy_amount
                                                        };

                                                        let token_name = if buy_is_usdc {
                                                            &sell_currency.symbol
                                                        } else {
                                                            &buy_currency.symbol
                                                        };

                                                        println!("Token {} price in USDC: {}", token_name, token_price_in_usdc);

                                                        if (buy_is_sol && sell_is_usdc) || (sell_is_sol && buy_is_usdc) {
                                                            last_sol_price_in_usdc = Some(
                                                                if buy_is_sol {
                                                                    sell_amount / buy_amount
                                                                } else {
                                                                    buy_amount / sell_amount
                                                                }
                                                            );
                                                            println!("Updated last SOL price in USDC => {:?}", last_sol_price_in_usdc);
                                                        }
                                                    } else if buy_is_sol || sell_is_sol {
                                                        if let Some(sol_price) = last_sol_price_in_usdc {
                                                            let token_in_sol = if buy_is_sol {
                                                                buy_amount / sell_amount
                                                            } else {
                                                                sell_amount / buy_amount
                                                            };

                                                            let token_name = if buy_is_sol {
                                                                &sell_currency.symbol
                                                            } else {
                                                                &buy_currency.symbol
                                                            };

                                                            let token_price_in_usdc = token_in_sol * sol_price;
                                                            println!(
                                                                "Token {} price in USDC (via SOL bridging): {}",
                                                                token_name, token_price_in_usdc
                                                            );
                                                        }
                                                    } else {
                                                        continue;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to decode DexParsedBlockMessage: {}", e);
                            }
                        }
                    }

                    consumer.commit_message(&msg, CommitMode::Async)?;
                }
                Err(e) => {
                    eprintln!("Error receiving message from Kafka: {}", e);
                }
            },
            Err(_) => {
                println!("No new messages within 10 seconds...");
            }
        }
    }
}
