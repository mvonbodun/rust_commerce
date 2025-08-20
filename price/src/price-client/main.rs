use bson::Decimal128;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use iso_currency::Currency;
use log::debug;
use offer_messages::{
    GetBestOfferPriceRequest, GetBestOfferPriceResponse, GetBestOfferPricesRequest,
    GetBestOfferPricesResponse, OfferCreateRequest, OfferCreateResponse, OfferDeleteRequest,
    OfferDeleteResponse, OfferGetRequest, OfferGetResponse,
};
use prost::Message;
use prost_types::Timestamp;
use rust_common::env_config::load_environment;
use rust_price::Offer;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub mod offer_messages {
    include!(concat!(env!("OUT_DIR"), "/offer_messages.rs"));
}

// Structs for JSON import (handles string dates and prices)
#[derive(Deserialize, Debug, Clone)]
pub struct OfferImport {
    pub sku: String,
    pub start_date: String,
    pub end_date: String,
    pub min_quantity: i32,
    pub max_quantity: Option<i32>,
    pub offer_prices: Vec<OfferPriceImport>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OfferPriceImport {
    pub price: String,
    pub currency: String,
}

// Convert import struct to domain model
impl OfferImport {
    fn to_offer(&self) -> Result<Offer, Box<dyn std::error::Error>> {
        Ok(Offer {
            id: None,
            sku: self.sku.clone(),
            start_date: DateTime::parse_from_rfc3339(&self.start_date)?.with_timezone(&Utc),
            end_date: DateTime::parse_from_rfc3339(&self.end_date)?.with_timezone(&Utc),
            min_quantity: self.min_quantity,
            max_quantity: self.max_quantity,
            offer_prices: self
                .offer_prices
                .iter()
                .map(|op| {
                    Ok(rust_price::OfferPrice {
                        price: Decimal128::from_str(&op.price)?,
                        currency: Currency::from_code(&op.currency)
                            .ok_or_else(|| format!("Invalid currency code: {}", op.currency))?,
                    })
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?,
        })
    }
}

// Helper function to convert Offer to OfferCreateRequest
fn offer_to_create_request(offer: &Offer) -> OfferCreateRequest {
    let ocr = OfferCreateRequest {
        sku: offer.sku.clone(),
        start_date: Some(Timestamp {
            seconds: offer.start_date.timestamp(),
            nanos: offer.start_date.timestamp_subsec_nanos() as i32,
        }),
        end_date: Some(Timestamp {
            seconds: offer.end_date.timestamp(),
            nanos: offer.end_date.timestamp_subsec_nanos() as i32,
        }),
        min_quantity: offer.min_quantity,
        max_quantity: offer.max_quantity,
        offer_prices: offer
            .offer_prices
            .iter()
            .map(|op| offer_messages::OfferPrice {
                price: op.price.to_string(),
                currency: op.currency.code().to_string(),
            })
            .collect(),
    };
    debug!("OfferCreateRequest: {ocr:?}");
    ocr
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    OfferCreate {
        #[arg(short, long)]
        sku: String,
        #[arg(short, long)]
        price: String,
        #[arg(short, long, default_value = "USD")]
        currency: String,
        #[arg(short = 'n', long, default_value = "1")]
        min_quantity: i32,
        #[arg(short = 'x', long)]
        max_quantity: Option<i32>,
    },
    OfferGet {
        #[arg(short, long)]
        id: String,
    },
    OfferDelete {
        #[arg(short, long)]
        id: String,
    },
    GetBestOfferPrice {
        #[arg(short, long)]
        sku: String,
        #[arg(short, long)]
        quantity: i32,
        #[arg(short, long, default_value = "USD")]
        currency: String,
        #[arg(short, long)]
        date: Option<String>,
    },
    GetBestOfferPrices {
        #[arg(short, long)]
        skus: String, // comma-separated list of SKUs
        #[arg(short, long)]
        quantity: i32,
        #[arg(short, long, default_value = "USD")]
        currency: String,
        #[arg(short, long)]
        date: Option<String>,
    },
    Import {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long, default_value = "false")]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    // Load environment variables
    load_environment();

    let cli = Cli::parse();

    // Get NATS URL from environment
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    // Connect to the nats server
    let client = async_nats::connect(&nats_url).await?;

    match &cli.command {
        Some(Commands::OfferCreate {
            sku,
            price,
            currency,
            min_quantity,
            max_quantity,
        }) => {
            // Validate currency
            Currency::from_code(currency)
                .ok_or_else(|| format!("Invalid currency: {currency}"))?;

            // Validate price format
            Decimal128::from_str(price).map_err(|_| format!("Invalid price format: {price}"))?;

            // Create an offer
            let offer_request = OfferCreateRequest {
                sku: sku.clone(),
                start_date: Some(Timestamp {
                    seconds: chrono::Utc::now().timestamp(),
                    nanos: 0,
                }),
                end_date: Some(Timestamp {
                    seconds: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp(),
                    nanos: 0,
                }),
                min_quantity: *min_quantity,
                max_quantity: *max_quantity,
                offer_prices: vec![offer_messages::OfferPrice {
                    price: price.clone(),
                    currency: currency.clone(),
                }],
            };

            let request_bytes = offer_request.encode_to_vec();

            println!("Sending create_offer request...");
            let response = client
                .request("offers.create_offer", request_bytes.into())
                .await?;

            let create_response = OfferCreateResponse::decode(&*response.payload)?;
            println!("Create response: {create_response:?}");
        }
        Some(Commands::OfferGet { id }) => {
            let get_request = OfferGetRequest { id: id.clone() };

            let request_bytes = get_request.encode_to_vec();

            println!("Sending get_offer request for ID: {id}");
            let response = client
                .request("offers.get_offer", request_bytes.into())
                .await?;

            let get_response = OfferGetResponse::decode(&*response.payload)?;
            println!("Get response: {get_response:?}");
        }
        Some(Commands::OfferDelete { id }) => {
            let delete_request = OfferDeleteRequest { id: id.clone() };

            let request_bytes = delete_request.encode_to_vec();

            println!("Sending delete_offer request for ID: {id}");
            let response = client
                .request("offers.delete_offer", request_bytes.into())
                .await?;

            let delete_response = OfferDeleteResponse::decode(&*response.payload)?;
            println!("Delete response: {delete_response:?}");
        }
        Some(Commands::GetBestOfferPrice {
            sku,
            quantity,
            currency,
            date,
        }) => {
            // Validate currency
            Currency::from_code(currency)
                .ok_or_else(|| format!("Invalid currency: {currency}"))?;

            let get_best_offer_request = GetBestOfferPriceRequest {
                sku: sku.clone(),
                quantity: *quantity,
                date: date.clone(),
                currency: currency.clone(),
            };

            let request_bytes = get_best_offer_request.encode_to_vec();

            println!(
                "Sending get_best_offer_price request for SKU: {sku} (quantity: {quantity}, currency: {currency})",
            );
            if let Some(date) = date {
                println!("  Date: {date}");
            }

            let response = client
                .request("offers.get_best_offer_price", request_bytes.into())
                .await?;

            let best_offer_response = GetBestOfferPriceResponse::decode(&*response.payload)?;

            if best_offer_response.found {
                if let Some(offer) = best_offer_response.offer {
                    println!("✅ Found best offer:");
                    println!(
                        "  Offer ID: {}",
                        offer.id.unwrap_or_else(|| "N/A".to_string())
                    );
                    println!("  SKU: {}", offer.sku);
                    println!("  Min Quantity: {}", offer.min_quantity);
                    println!(
                        "  Max Quantity: {}",
                        offer
                            .max_quantity
                            .map_or("N/A".to_string(), |q| q.to_string())
                    );
                    println!("  Prices:");
                    for price in &offer.offer_prices {
                        println!("    - {} {}", price.price, price.currency);
                    }
                } else {
                    println!("⚠️ Found offer but no details returned");
                }
            } else {
                println!(
                    "❌ No offer found for SKU: {sku} with quantity: {quantity} in currency: {currency}",
                );
            }
        }
        Some(Commands::GetBestOfferPrices {
            skus,
            quantity,
            currency,
            date,
        }) => {
            // Validate currency
            Currency::from_code(currency)
                .ok_or_else(|| format!("Invalid currency: {currency}"))?;

            // Parse comma-separated SKUs
            let sku_list: Vec<String> = skus
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if sku_list.is_empty() {
                return Err("No valid SKUs provided".into());
            }

            let get_best_offers_request = GetBestOfferPricesRequest {
                skus: sku_list.clone(),
                quantity: *quantity,
                date: date.clone(),
                currency: currency.clone(),
            };

            let request_bytes = get_best_offers_request.encode_to_vec();

            println!(
                "Sending get_best_offer_prices request for {} SKUs (quantity: {quantity}, currency: {currency})",
                sku_list.len(),
            );
            if let Some(date) = date {
                println!("  Date: {date}");
            }
            println!("  SKUs: {}", sku_list.join(", "));

            let response = client
                .request("offers.get_best_offer_prices", request_bytes.into())
                .await?;

            let best_offers_response = GetBestOfferPricesResponse::decode(&*response.payload)?;

            if let Some(status) = &best_offers_response.status {
                if status.code != 0 {
                    println!("❌ Error: {} (code: {})", status.message, status.code);
                    return Ok(());
                }
            }

            println!(
                "📊 Results for {} SKUs:",
                best_offers_response.sku_results.len()
            );

            let mut found_count = 0;
            let mut not_found_count = 0;

            for sku_result in &best_offers_response.sku_results {
                if sku_result.found {
                    found_count += 1;
                    if let Some(offer) = &sku_result.offer {
                        println!("✅ SKU: {}", sku_result.sku);
                        println!(
                            "   Offer ID: {}",
                            offer.id.as_ref().unwrap_or(&"N/A".to_string())
                        );
                        println!("   Min Quantity: {}", offer.min_quantity);
                        println!(
                            "   Max Quantity: {}",
                            offer
                                .max_quantity
                                .map_or("N/A".to_string(), |q| q.to_string())
                        );
                        println!("   Prices:");
                        for price in &offer.offer_prices {
                            println!("     - {} {}", price.price, price.currency);
                        }
                    } else {
                        println!("⚠️ SKU: {} - Found but no details", sku_result.sku);
                    }
                } else {
                    not_found_count += 1;
                    println!("❌ SKU: {} - No offer found", sku_result.sku);
                }
                println!(); // Empty line for readability
            }

            println!("Summary:");
            println!("  ✅ Found offers: {found_count}");
            println!("  ❌ No offers: {not_found_count}");
            println!(
                "  📊 Total SKUs: {}",
                best_offers_response.sku_results.len()
            );
        }
        Some(Commands::Import { file, dry_run }) => {
            println!("Importing offers from file: {file:?}");

            // Read and parse the JSON file
            let file_content = fs::read_to_string(file)?;

            // Try parsing as a single offer first
            let import_offers: Vec<OfferImport> =
                if let Ok(single_offer) = serde_json::from_str::<OfferImport>(&file_content) {
                    vec![single_offer]
                } else {
                    // Try parsing as an array of offers
                    serde_json::from_str::<Vec<OfferImport>>(&file_content)?
                };

            println!("Found {} offer(s) to import", import_offers.len());

            if *dry_run {
                println!("DRY RUN: Would import the following offers:");
                for (i, offer) in import_offers.iter().enumerate() {
                    println!(
                        "  {}. SKU: {} (min_qty: {}, max_qty: {:?})",
                        i + 1,
                        offer.sku,
                        offer.min_quantity,
                        offer.max_quantity
                    );
                    for price in &offer.offer_prices {
                        println!("    - {} {}", price.price, price.currency);
                    }
                }
                return Ok(());
            }

            let mut successful_imports = 0;
            let mut failed_imports = 0;

            for (i, import_offer) in import_offers.iter().enumerate() {
                println!(
                    "Importing offer {} of {}: SKU {} (min_qty: {}, max_qty: {:?})",
                    i + 1,
                    import_offers.len(),
                    import_offer.sku,
                    import_offer.min_quantity,
                    import_offer.max_quantity
                );

                // Convert import struct to domain model
                let offer = match import_offer.to_offer() {
                    Ok(offer) => offer,
                    Err(e) => {
                        println!("  ❌ Failed to parse offer SKU {}: {}", import_offer.sku, e);
                        failed_imports += 1;
                        continue;
                    }
                };

                let offer_request = offer_to_create_request(&offer);
                let request_bytes = offer_request.encode_to_vec();

                match client
                    .request("offers.create_offer", request_bytes.into())
                    .await
                {
                    Ok(response) => match OfferCreateResponse::decode(&*response.payload) {
                        Ok(create_response) => {
                            if let Some(status) = &create_response.status {
                                if status.code == offer_messages::Code::Ok as i32 {
                                    println!("  ✅ Successfully imported: SKU {}", offer.sku);
                                    successful_imports += 1;
                                } else {
                                    println!(
                                        "  ❌ Failed to import SKU {}: {} ({})",
                                        offer.sku, status.message, status.code
                                    );
                                    failed_imports += 1;
                                }
                            } else {
                                println!(
                                    "  ❌ Failed to import SKU {}: No status in response",
                                    offer.sku
                                );
                                failed_imports += 1;
                            }
                        }
                        Err(e) => {
                            println!(
                                "  ❌ Failed to decode response for SKU {}: {}",
                                offer.sku, e
                            );
                            failed_imports += 1;
                        }
                    },
                    Err(e) => {
                        println!("  ❌ Failed to send request for SKU {}: {}", offer.sku, e);
                        failed_imports += 1;
                    }
                }

                // Add a small delay to avoid overwhelming the service
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            println!("\nImport Summary:");
            println!("  ✅ Successful: {successful_imports}");
            println!("  ❌ Failed: {failed_imports}");
            println!("  📊 Total: {}", import_offers.len());
        }
        None => {
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}
