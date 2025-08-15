use std::env;
use log::info;

/// Load environment variables from environment-specific files
/// 
/// This function implements a layered approach to environment configuration:
/// 1. First loads from .env.local (for local development)
/// 2. Then loads from .env.production (for production deployment)
/// 3. Falls back to default .env if neither exists
/// 
/// The order ensures production settings override local ones when both exist.
pub fn load_environment() {
    let environment = env::var("RUST_ENV").unwrap_or_else(|_| "local".to_string());
    
    let env_file = match environment.as_str() {
        "production" => ".env.production",
        "local" | _ => ".env.local", // Default to local for any other value
    };
    
    match dotenvy::from_path(env_file) {
        Ok(_) => {
            info!("Loaded environment configuration from {}", env_file);
        }
        Err(_) => {
            // Fall back to default .env file
            match dotenvy::dotenv() {
                Ok(_) => info!("Loaded environment configuration from .env"),
                Err(e) => info!("No environment file found, using system environment: {}", e),
            }
        }
    }
}

