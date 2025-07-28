use clap::{Parser, Subcommand};
use catalog_messages::{
    ProductCreateRequest, ProductCreateResponse, ProductGetRequest, ProductGetResponse,
    ProductDeleteRequest, ProductDeleteResponse, ProductSearchRequest, ProductSearchResponse,
    ProductExportRequest, ProductExportResponse,
};
use log::debug;
use prost::Message;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use rust_catalog::Product;

pub mod catalog_messages {
    include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));
}

// Helper function to convert Product to ProductCreateRequest  
fn product_to_create_request(product: &Product) -> ProductCreateRequest {
    let pcr = ProductCreateRequest {
        name: product.name.clone(),
        long_description: product.long_description.clone(),
        brand: product.brand.clone(),
        slug: product.slug.clone(),
        product_ref: product.product_ref.clone(),
        product_type: product.product_type.clone(),
        seo_title: product.seo_title.clone(),
        seo_description: product.seo_description.clone(),
        seo_keywords: product.seo_keywords.clone(),
        display_on_site: product.display_on_site,
        tax_code: product.tax_code.clone(),
        related_products: product.related_products.clone(),
        reviews: product.reviews.as_ref().map(|r| catalog_messages::Reviews {
            bayesian_avg: r.bayesian_avg.into(),
            count: r.count,
            rating: r.rating,
        }),
        hierarchical_categories: product.hierarchical_categories.as_ref().map(|hc| catalog_messages::HierarchicalCategories {
            lvl0: hc.lvl0.clone(),
            lvl1: hc.lvl1.clone(),
            lvl2: hc.lvl2.clone(),
        }),
        list_categories: product.list_categories.clone(),
        defining_attributes: product.defining_attributes.clone(),
        descriptive_attributes: product.descriptive_attributes.clone(),
        default_variant: product.default_variant.clone(),
        variants: product.variants.iter().map(|v| catalog_messages::ProductVariant {
            sku: v.sku.clone(),
            defining_attributes: v.defining_attributes.clone().unwrap_or_default(),
            abbreviated_color: v.abbreviated_color.clone(),
            abbreviated_size: v.abbreviated_size.clone(),
            height: v.height,
            width: v.width,
            length: v.length,
            weight: v.weight,
            weight_unit: v.weight_unit.clone(),
            packaging: v.packaging.as_ref().map(|p| catalog_messages::Packaging {
                height: p.height,
                width: p.width,
                length: p.length,
                weight: p.weight,
                weight_unit: p.weight_unit.clone(),
            }),
            image_urls: v.image_urls.clone(),
        }).collect(),
    };
    debug!("ProductCreateRequest: {:?}", pcr);
    pcr
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ProductCreate {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        product_ref: String,
        #[arg(short, long)]
        brand: Option<String>,
    },
    ProductGet {
        #[arg(short, long)]
        id: String,
    },
    ProductDelete {
        #[arg(short, long)]
        id: String,
    },
    ProductSearch {
        #[arg(short, long)]
        query: Option<String>,
        #[arg(short, long)]
        category: Option<String>,
        #[arg(short, long)]
        brand: Option<String>,
    },
    Import {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long, default_value = "false")]
        dry_run: bool,
    },
    Export {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long, default_value = "50")]
        batch_size: i32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    
    let cli = Cli::parse();

    // Connect to the nats server
    let client = async_nats::connect("0.0.0.0:4222").await?;

    match &cli.command {
        Some(Commands::ProductCreate { name, product_ref, brand }) => {
            // Create a product
            let product_request = ProductCreateRequest {
                name: name.clone(),
                long_description: Some("Sample product description".to_string()),
                brand: brand.clone(),
                slug: Some(format!("{}-{}", name.to_lowercase().replace(" ", "-"), product_ref.to_lowercase())),
                product_ref: product_ref.clone(),
                product_type: None,
                seo_title: Some(name.clone()),
                seo_description: Some(format!("SEO description for {}", name)),
                seo_keywords: Some(format!("keywords, {}, product", name.to_lowercase())),
                display_on_site: true,
                tax_code: Some("txcd_99999999".to_string()),
                related_products: vec![],
                reviews: None,
                hierarchical_categories: None,
                list_categories: vec!["Sample Category".to_string()],
                defining_attributes: HashMap::new(),
                descriptive_attributes: HashMap::new(),
                default_variant: None,
                variants: vec![],
            };

            let request_bytes = product_request.encode_to_vec();
            
            println!("Sending create_product request...");
            let response = client
                .request("catalog.create_product", request_bytes.into())
                .await?;

            let create_response = ProductCreateResponse::decode(&*response.payload)?;
            println!("Create response: {:?}", create_response);
        }
        Some(Commands::ProductGet { id }) => {
            let get_request = ProductGetRequest {
                id: id.clone(),
            };

            let request_bytes = get_request.encode_to_vec();
            
            println!("Sending get_product request for ID: {}", id);
            let response = client
                .request("catalog.get_product", request_bytes.into())
                .await?;

            let get_response = ProductGetResponse::decode(&*response.payload)?;
            println!("Get response: {:?}", get_response);
        }
        Some(Commands::ProductDelete { id }) => {
            let delete_request = ProductDeleteRequest {
                id: id.clone(),
            };

            let request_bytes = delete_request.encode_to_vec();
            
            println!("Sending delete_product request for ID: {}", id);
            let response = client
                .request("catalog.delete_product", request_bytes.into())
                .await?;

            let delete_response = ProductDeleteResponse::decode(&*response.payload)?;
            println!("Delete response: {:?}", delete_response);
        }
        Some(Commands::ProductSearch { query, category, brand }) => {
            let categories = if let Some(cat) = category {
                vec![cat.clone()]
            } else {
                vec![]
            };

            let search_request = ProductSearchRequest {
                query: query.clone(),
                categories,
                brand: brand.clone(),
                limit: Some(10),
                offset: Some(0),
            };

            let request_bytes = search_request.encode_to_vec();
            
            println!("Sending search_products request...");
            let response = client
                .request("catalog.search_products", request_bytes.into())
                .await?;

            let search_response = ProductSearchResponse::decode(&*response.payload)?;
            println!("Search response: {:?}", search_response);
        }
        Some(Commands::Import { file, dry_run }) => {
            println!("Importing products from file: {:?}", file);
            
            // Read and parse the JSON file
            let file_content = fs::read_to_string(file)?;
            
            // Try parsing as a single product first
            let products: Vec<Product> = if let Ok(single_product) = serde_json::from_str::<Product>(&file_content) {
                vec![single_product]
            } else {
                // Try parsing as an array of products
                serde_json::from_str::<Vec<Product>>(&file_content)?
            };
            
            println!("Found {} product(s) to import", products.len());
            
            if *dry_run {
                println!("DRY RUN: Would import the following products:");
                for (i, product) in products.iter().enumerate() {
                    println!("  {}. {} (ref: {})", i + 1, product.name, product.product_ref);
                }
                return Ok(());
            }
            
            let mut successful_imports = 0;
            let mut failed_imports = 0;
            
            for (i, product) in products.iter().enumerate() {
                println!("Importing product {} of {}: {} (ref: {})", 
                    i + 1, products.len(), product.name, product.product_ref);
                
                let product_request = product_to_create_request(product);
                let request_bytes = product_request.encode_to_vec();
                
                match client.request("catalog.create_product", request_bytes.into()).await {
                    Ok(response) => {
                        match ProductCreateResponse::decode(&*response.payload) {
                            Ok(create_response) => {
                                if let Some(status) = &create_response.status {
                                    if status.code == catalog_messages::Code::Ok as i32 {
                                        println!("  ✅ Successfully imported: {}", product.name);
                                        successful_imports += 1;
                                    } else {
                                        println!("  ❌ Failed to import {}: {} ({})", 
                                            product.name, status.message, status.code);
                                        failed_imports += 1;
                                    }
                                } else {
                                    println!("  ❌ Failed to import {}: No status in response", product.name);
                                    failed_imports += 1;
                                }
                            }
                            Err(e) => {
                                println!("  ❌ Failed to decode response for {}: {}", product.name, e);
                                failed_imports += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("  ❌ Failed to send request for {}: {}", product.name, e);
                        failed_imports += 1;
                    }
                }
                
                // Add a small delay to avoid overwhelming the service
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            println!("\nImport Summary:");
            println!("  ✅ Successful: {}", successful_imports);
            println!("  ❌ Failed: {}", failed_imports);
            println!("  📊 Total: {}", products.len());
        }
        Some(Commands::Export { file, batch_size }) => {
            println!("Exporting all products to file: {:?}", file);
            println!("Using batch size: {}", batch_size);
            
            let mut all_products: Vec<Product> = Vec::new();
            let mut offset = 0i32;
            let mut total_exported = 0;
            
            loop {
                println!("Fetching batch starting at offset {}...", offset);
                
                let export_request = ProductExportRequest {
                    batch_size: Some(*batch_size),
                    offset: Some(offset),
                };

                let request_bytes = export_request.encode_to_vec();
                
                let response = client
                    .request("catalog.export_products", request_bytes.into())
                    .await?;

                let export_response = ProductExportResponse::decode(&*response.payload)?;
                
                match export_response.status {
                    Some(status) if status.code == catalog_messages::Code::Ok as i32 => {
                        let batch_count = export_response.products.len();
                        println!("Received {} products in this batch", batch_count);
                        
                        if batch_count == 0 {
                            println!("No more products to export");
                            break;
                        }
                        
                        // Convert proto products to domain products
                        for proto_product in export_response.products {
                            let product = Product {
                                id: proto_product.id,
                                name: proto_product.name,
                                long_description: proto_product.long_description,
                                brand: proto_product.brand,
                                slug: proto_product.slug,
                                product_ref: proto_product.product_ref,
                                product_type: proto_product.product_type,
                                seo_title: proto_product.seo_title,
                                seo_description: proto_product.seo_description,
                                seo_keywords: proto_product.seo_keywords,
                                display_on_site: proto_product.display_on_site,
                                tax_code: proto_product.tax_code,
                                related_products: proto_product.related_products,
                                reviews: proto_product.reviews.map(|r| rust_catalog::Reviews {
                                    bayesian_avg: r.bayesian_avg.into(),
                                    count: r.count,
                                    rating: r.rating,
                                }),
                                hierarchical_categories: proto_product.hierarchical_categories.map(|hc| rust_catalog::HierarchicalCategories {
                                    lvl0: hc.lvl0,
                                    lvl1: hc.lvl1,
                                    lvl2: hc.lvl2,
                                }),
                                list_categories: proto_product.list_categories,
                                created_at: proto_product.created_at.map(|ts| {
                                    use chrono::{DateTime, Utc};
                                    DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32).unwrap()
                                }),
                                updated_at: proto_product.updated_at.map(|ts| {
                                    use chrono::{DateTime, Utc};
                                    DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32).unwrap()
                                }),
                                created_by: proto_product.created_by,
                                updated_by: proto_product.updated_by,
                                defining_attributes: proto_product.defining_attributes,
                                descriptive_attributes: proto_product.descriptive_attributes,
                                default_variant: proto_product.default_variant,
                                variants: proto_product.variants.into_iter().map(|v| rust_catalog::ProductVariant {
                                    sku: v.sku,
                                    defining_attributes: Some(v.defining_attributes),
                                    abbreviated_color: v.abbreviated_color,
                                    abbreviated_size: v.abbreviated_size,
                                    height: v.height,
                                    width: v.width,
                                    length: v.length,
                                    weight: v.weight,
                                    weight_unit: v.weight_unit,
                                    packaging: v.packaging.map(|p| rust_catalog::Packaging {
                                        height: p.height,
                                        width: p.width,
                                        length: p.length,
                                        weight: p.weight,
                                        weight_unit: p.weight_unit,
                                    }),
                                    image_urls: v.image_urls,
                                }).collect(),
                            };
                            all_products.push(product);
                        }
                        
                        total_exported += batch_count;
                        offset += *batch_size;
                        
                        // If we got fewer than batch_size, we've reached the end
                        if (batch_count as i32) < *batch_size {
                            println!("Received fewer products than batch size, finished");
                            break;
                        }
                    }
                    Some(status) => {
                        println!("❌ Failed to export products: {} ({})", status.message, status.code);
                        break;
                    }
                    None => {
                        println!("❌ Invalid response from server");
                        break;
                    }
                }
                
                // Add a small delay to avoid overwhelming the service
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            // Write all products to file
            println!("Writing {} products to file...", all_products.len());
            let json_content = serde_json::to_string_pretty(&all_products)?;
            fs::write(file, json_content)?;
            
            println!("✅ Export completed!");
            println!("  📁 File: {:?}", file);
            println!("  📦 Total products: {}", total_exported);
        }
        None => {
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}
