use mongodb::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get MongoDB URL from environment or use default
    let mongodb_url = env::var("MONGODB_URL")
        .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string());
    
    println!("🧹 MongoDB Test Database Cleanup");
    println!("=================================");
    println!();
    
    // Mask password in URL for display
    let display_url = if mongodb_url.contains("@") {
        let parts: Vec<&str> = mongodb_url.split("://").collect();
        if parts.len() == 2 {
            let rest = parts[1];
            if let Some(at_pos) = rest.find('@') {
                let host_part = &rest[at_pos..];
                format!("{}://***:***{}", parts[0], host_part)
            } else {
                mongodb_url.clone()
            }
        } else {
            mongodb_url.clone()
        }
    } else {
        mongodb_url.clone()
    };
    
    println!("Connecting to: {}", display_url);
    println!();
    
    // Connect to MongoDB
    let client = Client::with_uri_str(&mongodb_url).await?;
    
    // List all databases
    let databases = client.list_database_names().await?;
    
    // Filter test databases
    let test_databases: Vec<String> = databases
        .iter()
        .filter(|name| name.starts_with("test_"))
        .cloned()
        .collect();
    
    if test_databases.is_empty() {
        println!("✨ No test databases found. MongoDB is clean!");
        return Ok(());
    }
    
    println!("Found {} test database(s):", test_databases.len());
    for db_name in &test_databases {
        println!("  - {}", db_name);
    }
    println!();
    
    // Ask for confirmation
    print!("Do you want to delete all test databases? (y/N): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cleanup cancelled");
        return Ok(());
    }
    
    // Delete test databases
    println!();
    println!("Cleaning up test databases...");
    
    for db_name in test_databases {
        print!("  Deleting {}...", db_name);
        io::stdout().flush()?;
        match client.database(&db_name).drop().await {
            Ok(_) => println!(" ✅"),
            Err(e) => println!(" ❌ Error: {}", e),
        }
    }
    
    println!();
    println!("✅ Cleanup complete!");
    
    // Show remaining databases
    println!();
    println!("Remaining databases:");
    let remaining = client.list_database_names().await?;
    for db_name in remaining {
        if !db_name.starts_with("test_") {
            println!("  - {}", db_name);
        }
    }
    
    Ok(())
}