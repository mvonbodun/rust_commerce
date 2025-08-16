use std::env;

/// Load environment variables using a layered approach
/// 
/// This function implements proper configuration precedence:
/// 1. **Base Layer**: Load .env (shared/default settings)
/// 2. **Environment Layer**: Load environment-specific file (.env.local, .env.production) 
/// 3. **System Layer**: System environment variables (highest priority)
/// 
/// Each layer can override values from previous layers, allowing for:
/// - Shared defaults in .env
/// - Environment-specific overrides in .env.local/.env.production  
/// - Runtime overrides via system environment
pub fn load_environment() {
    let mut loaded_files = Vec::new();
    
    // Layer 1: Load base .env file first (shared defaults)
    // Note: dotenvy won't override existing env vars, so we need to load in reverse priority order
    // We'll load .env.local FIRST, then .env as fallback for missing values
    
    let environment = env::var("RUST_ENV").unwrap_or_else(|_| "local".to_string());
    let env_specific_file = match environment.as_str() {
        "production" => ".env.production",
        "local" | _ => ".env.local", // Default to local for any other value
    };
    
    // Load environment-specific file FIRST (highest priority for file-based config)
    match dotenvy::from_path(env_specific_file) {
        Ok(_) => {
            loaded_files.push(env_specific_file.to_string());
        }
        Err(_) => {
            // Environment-specific file not found, that's okay
        }
    }
    
    // Then load base .env file (only for vars not already set by .env.local)
    match dotenvy::from_path(".env") {
        Ok(_) => {
            loaded_files.push(".env".to_string());
        }
        Err(_) => {
            // Try default dotenv behavior as fallback for base config
            match dotenvy::dotenv() {
                Ok(_) => loaded_files.push("default .env".to_string()),
                Err(_) => {} // No base config file found, continue
            }
        }
    }
    
    // Log what was loaded (in the order they were processed)
    if loaded_files.is_empty() {
        println!("🔧 No environment files found, using system environment only");
    } else {
        // Reverse the order for logging to show actual precedence
        loaded_files.reverse();
        println!("🔧 Loaded environment layers: {} (→ higher priority overrides lower)", loaded_files.join(" → "));
        println!("🔧 Final precedence: system env > {} > base defaults", loaded_files.first().unwrap_or(&"none".to_string()));
    }
}
