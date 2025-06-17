use serde::{Deserialize, Serialize};
use tracing::{info, error, warn, debug};
use mongodb::{Client, options::ClientOptions, Database, bson::{doc, DateTime as BsonDateTime}};
use redis::Client as RedisClient;
use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::SystemTime;

mod models;

pub use models::{Session, AccountingPacket};

// RADIUS Accounting packet types
const ACCT_STATUS_TYPE_START: u8 = 1;
const ACCT_STATUS_TYPE_STOP: u8 = 2;
const ACCT_STATUS_TYPE_INTERIM_UPDATE: u8 = 3;
const ACCT_STATUS_TYPE_ACCOUNTING_ON: u8 = 7;
const ACCT_STATUS_TYPE_ACCOUNTING_OFF: u8 = 8;

// RADIUS attribute types
const ATTR_ACCT_STATUS_TYPE: u8 = 40;
const ATTR_ACCT_SESSION_ID: u8 = 44;
const ATTR_ACCT_SESSION_TIME: u8 = 46;
const ATTR_ACCT_INPUT_OCTETS: u8 = 42;
const ATTR_ACCT_OUTPUT_OCTETS: u8 = 43;
const ATTR_ACCT_INPUT_PACKETS: u8 = 47;
const ATTR_ACCT_OUTPUT_PACKETS: u8 = 48;
const ATTR_ACCT_TERMINATE_CAUSE: u8 = 49;
const ATTR_USER_NAME: u8 = 1;
const ATTR_NAS_IP_ADDRESS: u8 = 4;
const ATTR_NAS_PORT: u8 = 5;

pub struct AccountingServer {
    config: Config,
    mongo_client: Client,
    mongo_db: Database,
    redis_client: RedisClient,
    socket: UdpSocket,
    secrets: HashMap<IpAddr, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mongo_url: String,
    pub mongo_db_name: String,
    pub redis_url: String,
    pub radius_bind_addr: String,
    pub quota_check_interval: u64,
    pub postgres_url: String,
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

        let postgres_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            warn!("DATABASE_URL not set, using default: postgres://postgres:postgres@localhost:5432/radius");
            "postgres://postgres:postgres@localhost:5432/radius".to_string()
        });

        let config = Self {
            mongo_url,
            mongo_db_name,
            redis_url,
            radius_bind_addr,
            quota_check_interval,
            postgres_url,
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

        // Initialize secrets map
        let secrets = HashMap::new();

        Ok(Self {
            config,
            mongo_client,
            mongo_db,
            redis_client,
            socket,
            secrets,
        })
    }

    async fn load_secrets(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Loading RADIUS secrets from database");
        
        // TODO: Implement loading secrets from PostgreSQL
        // For now, use a default secret for testing
        self.secrets.insert(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), "testing123".to_string());
        
        Ok(())
    }

    async fn handle_accounting_packet(&self, packet: &AccountingPacket, secret: &str) -> Result<(), Box<dyn std::error::Error>> {
        let status_type = packet.attributes.iter()
            .find(|attr| attr.typ == ATTR_ACCT_STATUS_TYPE)
            .and_then(|attr| attr.value.first().copied())
            .unwrap_or(0);

        match status_type {
            ACCT_STATUS_TYPE_START => {
                debug!("Processing Accounting-Start packet");
                self.create_session(packet).await?;
            }
            ACCT_STATUS_TYPE_STOP => {
                debug!("Processing Accounting-Stop packet");
                self.update_session(packet).await?;
            }
            ACCT_STATUS_TYPE_INTERIM_UPDATE => {
                debug!("Processing Accounting-Interim-Update packet");
                self.update_session(packet).await?;
            }
            ACCT_STATUS_TYPE_ACCOUNTING_ON => {
                debug!("Processing Accounting-On packet");
                // Handle NAS coming online
            }
            ACCT_STATUS_TYPE_ACCOUNTING_OFF => {
                debug!("Processing Accounting-Off packet");
                // Handle NAS going offline
            }
            _ => {
                warn!("Unknown accounting status type: {}", status_type);
            }
        }

        Ok(())
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

    async fn update_session(&self, packet: &AccountingPacket) -> Result<(), Box<dyn std::error::Error>> {
        let collection = self.mongo_db.collection::<Session>("sessions");
        
        // Find the session
        let filter = doc! { "session_id": &packet.session_id };
        let mut update = doc! {};

        // Update session attributes based on packet
        for attr in &packet.attributes {
            match attr.typ {
                ATTR_ACCT_SESSION_TIME => {
                    if let Ok(bytes) = attr.value[0..8].try_into() {
                        let time = u64::from_be_bytes(bytes);
                        update.insert("session_time", time as i64);
                    }
                }
                ATTR_ACCT_INPUT_OCTETS => {
                    if let Ok(bytes) = attr.value[0..8].try_into() {
                        let octets = u64::from_be_bytes(bytes);
                        update.insert("input_octets", octets as i64);
                    }
                }
                ATTR_ACCT_OUTPUT_OCTETS => {
                    if let Ok(bytes) = attr.value[0..8].try_into() {
                        let octets = u64::from_be_bytes(bytes);
                        update.insert("output_octets", octets as i64);
                    }
                }
                ATTR_ACCT_INPUT_PACKETS => {
                    if let Ok(bytes) = attr.value[0..8].try_into() {
                        let packets = u64::from_be_bytes(bytes);
                        update.insert("input_packets", packets as i64);
                    }
                }
                ATTR_ACCT_OUTPUT_PACKETS => {
                    if let Ok(bytes) = attr.value[0..8].try_into() {
                        let packets = u64::from_be_bytes(bytes);
                        update.insert("output_packets", packets as i64);
                    }
                }
                ATTR_ACCT_TERMINATE_CAUSE => {
                    if let Ok(cause) = String::from_utf8(attr.value.clone()) {
                        update.insert("termination_cause", cause);
                    }
                }
                _ => {}
            }
        }

        // If this is a stop packet, set the stop time
        if packet.attributes.iter().any(|attr| attr.typ == ATTR_ACCT_STATUS_TYPE && attr.value[0] == ACCT_STATUS_TYPE_STOP) {
            update.insert("stop_time", BsonDateTime::from_system_time(SystemTime::now()));
        }

        // Update the session
        if !update.is_empty() {
            collection.update_one(filter, doc! { "$set": update }).await?;
            info!("Updated session {} for user {}", packet.session_id, packet.username);
        }

        Ok(())
    }

    fn create_accounting_response(&self, request: &AccountingPacket) -> Vec<u8> {
        // Create Accounting-Response packet
        let mut response = Vec::new();
        
        // Code (5 = Accounting-Response)
        response.push(5);
        
        // Identifier (same as request)
        response.push(request.packet_type);
        
        // Length (will be updated)
        response.extend_from_slice(&[0u8, 0u8]);
        
        // Response Authenticator (will be calculated)
        response.extend_from_slice(&[0u8; 16]);
        
        // Update length
        let length = response.len() as u16;
        response[2] = (length >> 8) as u8;
        response[3] = length as u8;
        
        response
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting RADIUS accounting server on {}", self.config.radius_bind_addr);
        
        // Load secrets
        self.load_secrets().await?;
        
        let mut buf = [0u8; 4096];
        loop {
            let (len, src) = self.socket.recv_from(&mut buf).await?;
            debug!("Received {} bytes from {}", len, src);
            
            // Get secret for the NAS
            let secret = if let Some(secret) = self.secrets.get(&src.ip()) {
                secret
            } else {
                error!("No secret found for NAS {}", src.ip());
                continue;
            };

            // Parse the accounting packet
            if let Some(packet) = AccountingPacket::parse(&buf[..len]) {
                // Handle the accounting packet
                if let Err(e) = self.handle_accounting_packet(&packet, secret).await {
                    error!("Error handling accounting packet: {}", e);
                }

                // Send response
                let response = self.create_accounting_response(&packet);
                if let Err(e) = self.socket.send_to(&response, src).await {
                    error!("Failed to send response: {}", e);
                }
            } else {
                error!("Failed to parse accounting packet from {}", src);
            }
        }
    }
} 