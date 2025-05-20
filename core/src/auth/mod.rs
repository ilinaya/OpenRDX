use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, error};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use ipnetwork::IpNetwork;

mod radius_server;
mod models;

pub use radius_server::RadiusAuthServer;
pub use models::{NasDevice, Subscriber};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mongo_url: String,
    pub redis_url: String,
    pub radius_bind_addr: String,
    pub postgres_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading auth configuration from environment variables");
        
        // Default values for development
        let mongo_url = std::env::var("MONGO_URL").unwrap_or_else(|_| {
            warn!("MONGO_URL not set, using default: mongodb://localhost:27017");
            "mongodb://localhost:27017".to_string()
        });
        
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| {
            warn!("REDIS_URL not set, using default: redis://localhost:6379");
            "redis://localhost:6379".to_string()
        });
        
        let radius_bind_addr = std::env::var("RADIUS_BIND_ADDR").unwrap_or_else(|_| {
            warn!("RADIUS_BIND_ADDR not set, using default: 0.0.0.0:1812");
            "0.0.0.0:1812".to_string()
        });

        let postgres_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            warn!("DATABASE_URL not set, using default: postgres://postgres:postgres@localhost:5432/radius");
            "postgres://postgres:postgres@localhost:5432/radius".to_string()
        });

        let config = Self {
            mongo_url,
            redis_url,
            radius_bind_addr,
            postgres_url,
        };
        
        info!("Auth configuration loaded successfully");
        Ok(config)
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading auth configuration from file: {}", path.display());
        let contents = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        info!("Auth configuration loaded successfully from file");
        Ok(config)
    }
}

pub struct AuthServer {
    pub config: Config,
    db_pool: PgPool,
    nas_devices: HashMap<IpNetwork, NasDevice>,
}

impl AuthServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        debug!("Initializing AuthServer with config: {:?}", config);
        
        // Initialize database connection pool
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.postgres_url)
            .await?;
        debug!("Database connection pool initialized");

        let mut server = Self {
            config,
            db_pool,
            nas_devices: HashMap::new(),
        };

        // Load NAS devices
        server.load_nas_devices().await?;

        Ok(server)
    }

    async fn load_nas_devices(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Loading NAS devices from database");
        debug!("Executing NAS devices query");
        
        let nas_devices = sqlx::query_as!(
            NasDevice,
            r#"
            SELECT 
                nas_nas.id,
                nas_nas.name,
                nas_nas.is_active, radius_secret.source_subnets, radius_secret.secret
            FROM nas_nas
            LEFT JOIN radius_secret ON nas_nas.secret_id = radius_secret.id
            WHERE is_active = true
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        debug!("Query returned {} NAS devices", nas_devices.len());
        self.nas_devices.clear();
        
        for device in nas_devices {
            debug!("Processing NAS device: id={}, name={}, is_active={}", 
                device.id, device.name, device.is_active);
            if let Some(subnets) = &device.source_subnets {
                debug!("NAS device has source subnets: {:?}", subnets);
            }
            if let Some(secret) = &device.secret {
                debug!("NAS device has secret: {}", secret);
            }
            info!("Loaded NAS device: {}", device.id);
        }

        info!("Successfully loaded {} NAS devices", self.nas_devices.len());
        Ok(())
    }

    pub fn find_nas_device(&self, ip: IpNetwork) -> Option<&NasDevice> {
        self.nas_devices.get(&ip)
    }

    pub async fn refresh_nas_devices(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_nas_devices().await
    }
} 