pub mod queries;

use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use std::time::Duration;
use log::{info, warn, error};
use tokio::time::sleep;
use std::sync::Arc;
use anyhow::Result;

pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Parse the connection string into a Config
        // deadpool-postgres Config has individual fields, so we parse the URL and set them
        let url = url::Url::parse(database_url)
            .map_err(|e| anyhow::anyhow!("Invalid database URL: {}", e))?;
        
        let mut config = Config::new();
        
        if let Some(host) = url.host_str() {
            config.host = Some(host.to_string());
        }
        if let Some(port) = url.port() {
            config.port = Some(port);
        } else if url.scheme() == "postgres" || url.scheme() == "postgresql" {
            config.port = Some(5432); // Default PostgreSQL port
        }
        if !url.username().is_empty() {
            config.user = Some(url.username().to_string());
        }
        if let Some(password) = url.password() {
            config.password = Some(password.to_string());
        }
        if let Some(path) = url.path_segments() {
            if let Some(dbname) = path.last() {
                if !dbname.is_empty() {
                    config.dbname = Some(dbname.to_string());
                }
            }
        }
        
        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls)?;

        // Test connection with exponential backoff
        Self::test_connection_with_backoff(&pool).await?;

        info!("Database connection pool created successfully");
        Ok(Self { pool })
    }

    async fn test_connection_with_backoff(pool: &Pool) -> Result<()> {
        let mut attempt = 0;
        let max_attempts = 10;
        let mut delay = Duration::from_secs(1);

        loop {
            attempt += 1;
            match pool.get().await {
                Ok(client) => {
                    // Test query
                    match client.query("SELECT 1", &[]).await {
                        Ok(_) => {
                            info!("Database connection successful on attempt {}", attempt);
                            return Ok(());
                        }
                        Err(e) => {
                            error!("Database query failed: {}", e);
                            if attempt >= max_attempts {
                                return Err(anyhow::anyhow!("Failed to connect to database after {} attempts", max_attempts));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Database connection attempt {} failed: {}", attempt, e);
                    if attempt >= max_attempts {
                        return Err(anyhow::anyhow!("Failed to connect to database after {} attempts", max_attempts));
                    }
                }
            }

            info!("Retrying database connection in {:?}...", delay);
            sleep(delay).await;
            delay = Duration::from_secs(delay.as_secs() * 2).min(Duration::from_secs(60)); // Exponential backoff, max 60s
        }
    }
}

// Reconnect task for handling connection failures
pub async fn start_reconnect_task(pool: Arc<Pool>) {
    tokio::spawn(async move {
        let mut last_check = std::time::Instant::now();
        let check_interval = Duration::from_secs(30);

        loop {
            sleep(check_interval).await;

            // Check if we can get a connection
            match pool.get().await {
                Ok(client) => {
                    match client.query("SELECT 1", &[]).await {
                        Ok(_) => {
                            if last_check.elapsed() > Duration::from_secs(60) {
                                info!("Database connection healthy");
                                last_check = std::time::Instant::now();
                            }
                        }
                        Err(e) => {
                            error!("Database connection lost: {}", e);
                            warn!("Attempting to reconnect...");
                            // The pool will automatically try to reconnect on next get()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    warn!("Pool will attempt to reconnect on next request");
                }
            }
        }
    });
}

