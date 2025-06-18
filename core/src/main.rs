use dotenv::dotenv;
use std::path::Path;
use tracing::{info, error, debug, warn};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
use std::sync::Arc;
use tokio::signal;
use std::time::Duration;

mod auth;
mod accounting;

#[derive(Debug)]
enum ServiceType {
    Auth,
    Accounting,
}

impl std::str::FromStr for ServiceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auth" => Ok(ServiceType::Auth),
            "acct" => Ok(ServiceType::Accounting),
            _ => Err(format!("Invalid service type: {}", s)),
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown...");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    if let Err(e) = dotenv() {
        warn!("Failed to load .env file: {}", e);
    }

    // Initialize logging with more detailed configuration
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap_or_else(|_| {
            warn!("Failed to parse log filter, using default");
            EnvFilter::new("debug")
        });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    // Determine which service to run based on environment
    let service_type = std::env::var("SERVICE_TYPE")
        .unwrap_or_else(|_| "auth".to_string())
        .parse::<ServiceType>()
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

    debug!("Starting service type: {:?}", service_type);

    // Create shutdown signal handler
    let shutdown = shutdown_signal();

    match service_type {
        ServiceType::Auth => {
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

            // Run the server until shutdown signal is received
            tokio::select! {
                result = radius_server.run() => {
                    if let Err(e) = result {
                        error!("RADIUS server error: {}", e);
                        return Err(e.into());
                    }
                }
                _ = shutdown => {
                    info!("Shutdown signal received, stopping RADIUS server");
                }
            }
        }
        ServiceType::Accounting => {
            // Load accounting configuration
            let config = accounting::Config::from_env()?;

            info!("Starting RADIUS accounting service");
            info!("Configuration loaded: {:?}", config);

            // Create and start the accounting server
            let mut server = accounting::AccountingServer::new(config).await?;
            
            // Run the server until shutdown signal is received
            tokio::select! {
                result = server.start() => {
                    if let Err(e) = result {
                        error!("Accounting server error: {}", e);
                        return Err(e.into());
                    }
                }
                _ = shutdown => {
                    info!("Shutdown signal received, stopping accounting server");
                }
            }
        }
    }

    // Give some time for graceful shutdown
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Shutdown complete");

    Ok(())
}