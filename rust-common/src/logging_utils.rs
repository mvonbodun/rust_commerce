use log::{debug, error, info, warn};
use std::time::Instant;

/// Performance timing utility for monitoring operation durations
pub struct OperationTimer {
    start: Instant,
    operation: String,
}

impl OperationTimer {
    /// Create a new timer for the given operation
    pub fn new(operation: &str) -> Self {
        debug!("⏱️  Starting operation: {operation}");
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    /// Manually log elapsed time with specified level
    pub fn log_elapsed(&self, level: &str) {
        let elapsed = self.start.elapsed();
        match level {
            "debug" => debug!("⏱️  {} completed in {:?}", self.operation, elapsed),
            "info" => info!("⏱️  {} completed in {:?}", self.operation, elapsed),
            "warn" => warn!(
                "⏱️  {} took {:?} (slower than expected)",
                self.operation, elapsed
            ),
            "error" => error!("⏱️  {} failed after {:?}", self.operation, elapsed),
            _ => debug!("⏱️  {} completed in {:?}", self.operation, elapsed),
        }
    }

    /// Get elapsed time without logging
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        // Only log if operation took longer than 1ms to avoid noise
        if elapsed.as_millis() > 1 {
            debug!("⏱️  {} completed in {:?}", self.operation, elapsed);
        }
    }
}

/// Enhanced error context trait for better debugging
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context(self, context: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        self.map_err(|e| {
            error!("❌ {context}: {e}");
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })
    }
}

/// Utility for masking sensitive information in URLs
pub fn mask_sensitive_url(url: &str) -> String {
    // Simple pattern matching to mask passwords in MongoDB URLs
    if url.contains("://") && url.contains("@") {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() == 2 {
            let scheme = parts[0];
            let rest = parts[1];

            if let Some(at_pos) = rest.find('@') {
                let auth_part = &rest[..at_pos];
                let host_part = &rest[at_pos..];

                // Mask password if present
                if let Some(colon_pos) = auth_part.find(':') {
                    let username = &auth_part[..colon_pos];
                    return format!("{scheme}://{username}:***{host_part}");
                }
            }
        }
    }
    url.to_string()
}

/// Network debugging utilities for Fly.io environments
pub async fn debug_dns_resolution(hostname: &str) {
    info!("🔍 Debugging DNS resolution for: {hostname}");

    // Extract hostname from URL if needed
    let host_to_resolve = if hostname.starts_with("nats://") {
        hostname
            .replace("nats://", "")
            .split(':')
            .next()
            .unwrap_or(hostname)
            .to_string()
    } else {
        hostname.to_string()
    };

    // Try to resolve DNS
    match tokio::net::lookup_host(&format!("{host_to_resolve}:4222")).await {
        Ok(addrs) => {
            info!("✅ DNS resolution successful for {host_to_resolve}");
            for addr in addrs {
                debug!("   📍 Resolved to: {addr}");
            }
        }
        Err(e) => {
            error!("❌ DNS resolution failed for {host_to_resolve}: {e}");
        }
    }
}

/// Health monitoring utilities
pub struct HealthMonitor {
    pub mongodb_client: mongodb::Client,
    pub nats_client: async_nats::Client,
}

impl HealthMonitor {
    pub fn new(mongodb_client: mongodb::Client, nats_client: async_nats::Client) -> Self {
        Self {
            mongodb_client,
            nats_client,
        }
    }

    /// Start periodic health checks
    pub fn start_health_checks(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                self.perform_health_check().await;
            }
        });
    }

    /// Perform a single health check
    async fn perform_health_check(&self) {
        // MongoDB health check
        match self.mongodb_client.list_database_names().await {
            Ok(_) => debug!("💓 MongoDB health check: OK"),
            Err(e) => error!("💔 MongoDB health check failed: {e}"),
        }

        // NATS health check
        match self.nats_client.connection_state() {
            async_nats::connection::State::Connected => debug!("💓 NATS health check: OK"),
            state => warn!("💔 NATS connection state: {state:?}"),
        }
    }
}

/// Setup graceful shutdown signal handlers
pub async fn setup_signal_handlers() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::signal;

    tokio::spawn(async {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();

        tokio::select! {
            _ = sigterm.recv() => {
                info!("🛑 Received SIGTERM, initiating graceful shutdown...");
            }
            _ = sigint.recv() => {
                info!("🛑 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
            }
        }

        info!("🧹 Cleaning up resources...");
        // Add cleanup logic here
        info!("✅ Graceful shutdown completed");
        std::process::exit(0);
    });

    Ok(())
}

/// Generic service dependency validation
///
/// This is a generic version that can be customized by each service.
/// Services should implement their own validation logic by calling this
/// and adding service-specific checks.
pub async fn validate_dependencies(
    mongo_client: &mongodb::Client,
    nats_client: &async_nats::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _timer = OperationTimer::new("dependency validation");
    info!("🔍 Validating service dependencies...");

    // Validate MongoDB connection
    let databases = mongo_client
        .list_database_names()
        .await
        .with_context("Failed to list MongoDB databases")?;
    debug!(
        "✅ MongoDB connection validated, {} databases available",
        databases.len()
    );

    // Test NATS subscription capability
    match nats_client
        .queue_subscribe("health_check", "test_queue".to_string())
        .await
    {
        Ok(_) => {
            debug!("✅ NATS subscription test successful");
        }
        Err(e) => {
            return Err(format!("NATS subscription test failed: {e}").into());
        }
    }

    info!("✅ Core dependencies validated successfully");
    Ok(())
}

//
// Log Level Guidelines:
// - ERROR: Critical failures that require immediate attention
// - WARN:  Unexpected conditions that don't stop operation
// - INFO:  Important operational events (startup, shutdown, major operations)
// - DEBUG: Detailed flow information for development/troubleshooting
// - TRACE: Very detailed information, including data dumps
//
// Service-specific recommendations:
// - Use DEBUG for development, INFO for production
// - mongodb: Use WARN to avoid noise, DEBUG only when investigating DB issues
// - async_nats: Use WARN to avoid connection noise, DEBUG for message tracing
// - hyper/h2: Use ERROR only, these are very noisy in debug mode
//
