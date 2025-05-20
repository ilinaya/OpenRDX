use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use mongodb::{Client, options::ClientOptions, Database};
use redis::Client as RedisClient;
use tokio::net::UdpSocket;
use chrono::Utc;

mod models;

pub use models::{Session, Quota, AccountingPacket, AccountingAttribute};

pub struct AccountingServer {
    config: Config,
    mongo_client: Client,
    mongo_db: Database,
    redis_client: RedisClient,
    socket: UdpSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mongo_url: String,
    pub mongo_db_name: String,
    pub redis_url: String,
    pub radius_bind_addr: String,
    pub quota_check_interval: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading accounting configuration from environment variables");
        
        // Default values for development
        let mongo_url = std::env::var("MONGO_URL").unwrap_or_else(|_| {
            warn!("MONGO_URL not set, using default: mongodb://localhost:27017");
            "mongodb://localhost:27017".to_string()
        });
        
        let mongo_db_name = std::env::var("MONGO_DB_NAME").unwrap_or_else(|_| {
            warn!("MONGO_DB_NAME not set, using default: radius");
            "radius".to_string()
        });
        
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| {
            warn!("REDIS_URL not set, using default: redis://localhost:6379");
            "redis://localhost:6379".to_string()
        });
        
        let radius_bind_addr = std::env::var("RADIUS_BIND_ADDR").unwrap_or_else(|_| {
            warn!("RADIUS_BIND_ADDR not set, using default: 0.0.0.0:1813");
            "0.0.0.0:1813".to_string()
        });
        
        let quota_check_interval = std::env::var("QUOTA_CHECK_INTERVAL")
            .unwrap_or_else(|_| {
                warn!("QUOTA_CHECK_INTERVAL not set, using default: 300");
                "300".to_string()
            })
            .parse()
            .unwrap_or(300);

        let config = Self {
            mongo_url,
            mongo_db_name,
            redis_url,
            radius_bind_addr,
            quota_check_interval,
        };
        
        info!("Accounting configuration loaded successfully");
        Ok(config)
    }
}

impl AccountingServer {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize MongoDB client
        let mongo_options = ClientOptions::parse(&config.mongo_url).await?;
        let mongo_client = Client::with_options(mongo_options)?;
        let mongo_db = mongo_client.database(&config.mongo_db_name);

        // Initialize Redis client
        let redis_client = RedisClient::open(config.redis_url.clone())?;

        // Bind UDP socket
        let socket = UdpSocket::bind(&config.radius_bind_addr).await?;

        Ok(Self {
            config,
            mongo_client,
            mongo_db,
            redis_client,
            socket,
        })
    }

    async fn create_session(&self, packet: &AccountingPacket) -> Result<(), Box<dyn std::error::Error>> {
        let session = Session {
            session_id: packet.session_id.clone(),
            username: packet.username.clone(),
            nas_ip: packet.nas_ip.clone(),
            nas_port: packet.nas_port,
            start_time: packet.timestamp,
            stop_time: None,
            input_octets: 0,
            output_octets: 0,
            input_packets: 0,
            output_packets: 0,
            session_time: 0,
            termination_cause: None,
        };

        // Insert session into MongoDB
        let collection = self.mongo_db.collection::<Session>("sessions");
        collection.insert_one(session).await?;
        info!("Created new session for user {} from NAS {}", packet.username, packet.nas_ip);
        
        Ok(())
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting RADIUS accounting server on {}", self.config.radius_bind_addr);
        
        let mut buf = [0u8; 4096];
        loop {
            let (len, src) = self.socket.recv_from(&mut buf).await?;
            info!("Received {} bytes from {}", len, src);
            
            // TODO: Parse RADIUS accounting packet
            // For now, create a dummy packet for testing
            let packet = AccountingPacket {
                packet_type: 4, // Accounting-Request
                session_id: format!("test-session-{}", Utc::now().timestamp()),
                username: "test_user".to_string(),
                nas_ip: src.ip().to_string(),
                nas_port: src.port() as u32,
                timestamp: Utc::now(),
                attributes: Vec::new(),
            };

            // Create new session in MongoDB
            if let Err(e) = self.create_session(&packet).await {
                error!("Failed to create session: {}", e);
            }
        }
    }
} 