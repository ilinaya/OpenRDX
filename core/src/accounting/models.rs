use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub username: String,
    pub nas_ip: String,
    pub nas_port: u32,
    pub start_time: DateTime<Utc>,
    pub stop_time: Option<DateTime<Utc>>,
    pub input_octets: u64,
    pub output_octets: u64,
    pub input_packets: u64,
    pub output_packets: u64,
    pub session_time: u64,
    pub termination_cause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    pub username: String,
    pub data_limit: u64,
    pub time_limit: u64,
    pub used_data: u64,
    pub used_time: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingPacket {
    pub packet_type: u8,
    pub session_id: String,
    pub username: String,
    pub nas_ip: String,
    pub nas_port: u32,
    pub timestamp: DateTime<Utc>,
    pub attributes: Vec<AccountingAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingAttribute {
    pub typ: u8,
    pub value: Vec<u8>,
} 