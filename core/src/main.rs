use dotenv::dotenv;
use std::path::Path;
use tracing::{info, error, Level};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan, FmtSubscriber};

mod auth;
mod accounting;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging with more detailed configuration
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_env_filter(EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info,radius_server=debug"))?)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_level(true)
        .with_ansi(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Determine which service to run based on the environment
    let service_type = std::env::var("SERVICE_TYPE").unwrap_or_else(|_| "auth".to_string());

    match service_type.as_str() {
        "auth" => {
            // Load auth configuration
            let config = if let Ok(config_path) = std::env::var("CONFIG_PATH") {
                auth::Config::from_file(Path::new(&config_path))?
            } else {
                auth::Config::from_env()?
            };

            info!("Starting RADIUS authorization service");
            info!("Configuration loaded: {:?}", config);

            // Create and start the auth server
            let server = auth::AuthServer::new(config).await?;
            
            // Start the RADIUS server
            let radius_server = auth::RadiusAuthServer::new(server.config.radius_bind_addr.clone());
            radius_server.run().await?;
        }
        "acct" => {
            // Load accounting configuration
            let config = accounting::Config::from_env()?;

            info!("Starting RADIUS accounting service");
            info!("Configuration loaded: {:?}", config);

            // Create and start the accounting server
            let server = accounting::AccountingServer::new(config).await?;
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