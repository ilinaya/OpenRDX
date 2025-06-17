use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, error};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use sqlx::types::JsonValue;
use ipnetwork::IpNetwork;
use std::net::IpAddr;

mod radius_server;
mod models;

pub use radius_server::RadiusAuthServer;
pub use models::{NasDevice};

#[derive(Debug, Clone)]
struct SecretInfo {
    secret: String,
    subnets: Vec<IpNetwork>,
}

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
    secrets: HashMap<IpNetwork, SecretInfo>,
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
            secrets: HashMap::new(),
        };

        // Load NAS devices and secrets
        server.load_nas_devices().await?;
        server.load_secrets().await?;

        Ok(server)
    }

    async fn load_secrets(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Loading RADIUS secrets from database");
        debug!("Executing secrets query");
        
        #[derive(sqlx::FromRow)]
        struct SecretRow {
            id: i64,
            secret: Option<String>,
            source_subnets: Option<JsonValue>,
        }

        let secrets = sqlx::query_as::<_, SecretRow>(
            r#"
            SELECT 
                id,
                secret,
                source_subnets
            FROM radius_secret
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        debug!("Query returned {} secrets", secrets.len());
        self.secrets.clear();
        
        for secret in secrets {
            debug!("Processing secret: id={}", secret.id);
            
            if let Some(subnets_json) = secret.source_subnets {
                debug!("Raw subnets JSON: {:?}", subnets_json);
                let subnets: Vec<String> = serde_json::from_value(subnets_json)?;
                debug!("Parsed subnets: {:?}", subnets);
                
                let mut ip_networks = Vec::new();
                
                for subnet in subnets {
                    match subnet.parse::<IpNetwork>() {
                        Ok(network) => {
                            ip_networks.push(network);
                            debug!("Added subnet: {} (prefix: {}) for secret ID {}", 
                                network, network.prefix(), secret.id);
                        }
                        Err(e) => {
                            warn!("Invalid subnet format '{}' for secret ID {}: {}", subnet, secret.id, e);
                        }
                    }
                }
                
                if let Some(secret_str) = secret.secret {
                    debug!("Found secret for ID {}: {}", secret.id, secret_str);
                    let secret_info = SecretInfo {
                        secret: secret_str,
                        subnets: ip_networks.clone(),
                    };
                    
                    // Store the secret for each subnet
                    for network in &ip_networks {
                        debug!("Mapping subnet {} (prefix: {}) to secret ID {}", 
                            network, network.prefix(), secret.id);
                        self.secrets.insert(*network, secret_info.clone());
                    }
                } else {
                    warn!("No secret found for ID {}", secret.id);
                }
            } else {
                warn!("No subnets found for secret ID {}", secret.id);
            }
        }

        // Print final mapping for verification
        debug!("Final subnet-secret mappings:");
        for (network, secret_info) in &self.secrets {
            debug!("Subnet: {} (prefix: {}) -> Secret: {}", 
                network, network.prefix(), secret_info.secret);
        }

        info!("Successfully loaded {} subnet-secret mappings", self.secrets.len());
        Ok(())
    }

    fn parse_ip(ip: &str) -> Option<IpAddr> {
        match ip.parse::<IpAddr>() {
            Ok(addr) => {
                debug!("Successfully parsed IP address: {}", addr);
                Some(addr)
            }
            Err(e) => {
                error!("Failed to parse IP address '{}': {}", ip, e);
                None
            }
        }
    }

    pub fn find_secret_for_ip(&self, ip: impl Into<IpAddr>) -> Option<&str> {
        let ip_addr = ip.into();
        debug!("Finding secret for IP: {}", ip_addr);
        debug!("IP type: {:?}", ip_addr);
        debug!("Available subnets: {:?}", self.secrets.keys().collect::<Vec<_>>());
        
        // Find all matching subnets
        let matching_subnets: Vec<&IpNetwork> = self.secrets.keys()
            .filter(|network| {
                let contains = network.contains(ip_addr);
                debug!("Checking subnet {} against IP {}: {}", network, ip_addr, contains);
                debug!("Subnet type: {:?}, IP type: {:?}", network.ip(), ip_addr);
                debug!("Subnet prefix: {}, Network: {}", network.prefix(), network);
                contains
            })
            .collect();
        
        if matching_subnets.is_empty() {
            debug!("No matching subnet found for IP: {}", ip_addr);
            return None;
        }
        
        debug!("Found matching subnets: {:?}", matching_subnets);
        
        // Find the most specific (smallest) subnet
        let most_specific = matching_subnets.iter()
            .min_by_key(|network| {
                let prefix = network.prefix();
                debug!("Subnet {} has prefix {}", network, prefix);
                prefix
            })
            .unwrap();
        
        debug!("Selected most specific subnet: {} (prefix: {})", 
            most_specific, most_specific.prefix());
        
        // Get the secret for the most specific subnet
        let secret = self.secrets.get(most_specific)
            .map(|info| info.secret.as_str());
        
        debug!("Found secret for IP {}: {:?}", ip_addr, secret);
        secret
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
                nas_nas.is_active 
            FROM nas_nas
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

    pub async fn refresh_secrets(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_secrets().await
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.db_pool
    }
} 