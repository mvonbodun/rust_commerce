use clap::{Parser, Subcommand};
use catalog_messages::{
    ProductCreateRequest, ProductCreateResponse, ProductGetRequest, ProductGetResponse,
    ProductDeleteRequest, ProductDeleteResponse, ProductSearchRequest, ProductSearchResponse,
};
use prost::Message;
use std::collections::HashMap;

pub mod catalog_messages {
    include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    
    let cli = Cli::parse();

    // Connect to the nats server
    let client = async_nats::connect("0.0.0.0:4222").await?;

    match &cli.command {
        Some(Commands::ProductCreate { name, brand }) => {
            // Create a product
            let product_request = ProductCreateRequest {
                name: name.clone(),
                long_description: Some("Sample product description".to_string()),
                brand: brand.clone(),
                product_ref: Some(12345), // Changed to int32
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
        None => {
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}
