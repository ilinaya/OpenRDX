use dotenv::dotenv;
use std::path::Path;
use tracing::{info, error, debug};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
use std::sync::Arc;

mod auth;
mod accounting;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging with more detailed configuration
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    // Determine which service to run based on environment
    let service_type = std::env::var("SERVICE_TYPE").unwrap_or_else(|_| "auth".to_string());
    debug!("Starting service type: {}", service_type);

    match service_type.as_str() {
        "auth" => {
            // Load auth configuration
            let config = if let Ok(config_path) = std::env::var("CONFIG_PATH") {
                debug!("Loading config from file: {}", config_path);
                auth::Config::from_file(Path::new(&config_path))?
            } else {
                debug!("Loading config from environment variables");
                auth::Config::from_env()?
            };

            info!("Starting RADIUS authorization service");
            debug!("Configuration loaded: {:?}", config);
            debug!("RADIUS bind address: {}", config.radius_bind_addr);

            // Create and start the auth server
            let auth_server = Arc::new(auth::AuthServer::new(config.clone()).await?);
            
            // Start the RADIUS server
            debug!("Initializing RADIUS server");
            let radius_server = auth::RadiusAuthServer::new(config.radius_bind_addr, auth_server).await?;
            debug!("Starting RADIUS server loop");
            radius_server.run().await?;
        }
        "acct" => {
            // Load accounting configuration
            let config = accounting::Config::from_env()?;

            info!("Starting RADIUS accounting service");
            info!("Configuration loaded: {:?}", config);

            // Create and start the accounting server
            let mut server = accounting::AccountingServer::new(config).await?;
            if let Err(e) = server.start().await {
                error!("Accounting server error: {}", e);
                return Err(e.into());
            }
        }
        _ => {
            error!("Invalid service type: {}", service_type);
            return Err("Invalid service type".into());
        }
    }

    Ok(())
}