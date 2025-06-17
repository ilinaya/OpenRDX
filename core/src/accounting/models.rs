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

impl AccountingPacket {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 20 { return None; }
        
        //let code = data[0];
        let identifier = data[1];
        let length = u16::from_be_bytes([data[2], data[3]]);
        if data.len() < length as usize { return None; }
        
        let mut pos = 20;
        let mut attributes = Vec::new();
        let mut session_id = String::new();
        let mut username = String::new();
        let mut nas_ip = String::new();
        let mut nas_port = 0u32;

        while pos < length as usize {
            if data.len() < pos + 2 { break; }
            let typ = data[pos];
            let len = data[pos + 1] as usize;
            if data.len() < pos + len { break; }
            
            let value = data[pos + 2..pos + len].to_vec();
            attributes.push(AccountingAttribute { typ, value: value.clone() });

            // Extract common attributes
            match typ {
                1 => { // User-Name
                    if let Ok(name) = String::from_utf8(value) {
                        username = name;
                    }
                }
                4 => { // NAS-IP-Address
                    if value.len() == 4 {
                        nas_ip = format!("{}.{}.{}.{}", value[0], value[1], value[2], value[3]);
                    }
                }
                5 => { // NAS-Port
                    if value.len() == 4 {
                        nas_port = u32::from_be_bytes([value[0], value[1], value[2], value[3]]);
                    }
                }
                44 => { // Acct-Session-Id
                    if let Ok(id) = String::from_utf8(value) {
                        session_id = id;
                    }
                }
                _ => {}
            }

            pos += len;
        }

        Some(Self {
            packet_type: identifier,
            session_id,
            username,
            nas_ip,
            nas_port,
            timestamp: Utc::now(),
            attributes,
        })
    }
} 