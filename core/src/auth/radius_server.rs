use tokio::net::UdpSocket;
use tracing::{info, debug, error};
use std::net::IpAddr;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NasSecret {
    pub subnet: String, // CIDR notation
    pub secret: String,
}

pub struct NasSecretManager {
    secrets: Vec<NasSecret>,
}

impl NasSecretManager {
    pub fn new() -> Self {
        // For now, use a static list. Replace with DB loading as needed.
        let secrets = vec![
            NasSecret { subnet: "192.168.1.0/24".to_string(), secret: "sharedsecret1".to_string() },
            NasSecret { subnet: "10.0.0.0/8".to_string(), secret: "sharedsecret2".to_string() },
        ];
        Self { secrets }
    }
    pub fn find_secret(&self, ip: IpAddr) -> Option<&NasSecret> {
        for entry in &self.secrets {
            if let Ok(net) = entry.subnet.parse::<ipnetwork::IpNetwork>() {
                if net.contains(ip) {
                    return Some(entry);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct RadiusAttribute {
    pub typ: u8,
    pub value: Vec<u8>,
}

impl RadiusAttribute {
    pub fn parse(data: &[u8]) -> Option<(Self, usize)> {
        if data.len() < 2 { return None; }
        let typ = data[0];
        let len = data[1] as usize;
        if len < 2 || data.len() < len { return None; }
        let value = data[2..len].to_vec();
        Some((Self { typ, value }, len))
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(2 + self.value.len());
        out.push(self.typ);
        out.push((self.value.len() + 2) as u8);
        out.extend_from_slice(&self.value);
        out
    }
}

#[derive(Debug, Clone)]
pub struct RadiusPacket {
    pub code: u8,
    pub identifier: u8,
    pub length: u16,
    pub authenticator: [u8; 16],
    pub attributes: Vec<RadiusAttribute>,
}

impl RadiusPacket {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 20 { return None; }
        let code = data[0];
        let identifier = data[1];
        let length = u16::from_be_bytes([data[2], data[3]]);
        if data.len() < length as usize { return None; }
        let mut authenticator = [0u8; 16];
        authenticator.copy_from_slice(&data[4..20]);
        let mut pos = 20;
        let mut attributes = Vec::new();
        while pos < length as usize {
            if let Some((attr, used)) = RadiusAttribute::parse(&data[pos..]) {
                attributes.push(attr);
                pos += used;
            } else {
                break;
            }
        }
        Some(Self { code, identifier, length, authenticator, attributes })
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.length as usize);
        out.push(self.code);
        out.push(self.identifier);
        out.extend_from_slice(&(self.length.to_be_bytes()));
        out.extend_from_slice(&self.authenticator);
        for attr in &self.attributes {
            out.extend_from_slice(&attr.encode());
        }
        out
    }
}

pub struct RadiusAuthServer {
    bind_addr: String,
    nas_manager: NasSecretManager,
}

impl RadiusAuthServer {
    pub fn new(bind_addr: String) -> Self {
        Self {
            bind_addr,
            nas_manager: NasSecretManager::new(),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(&self.bind_addr).await?;
        info!("RADIUS Auth server listening on {}", self.bind_addr);

        let mut buf = [0u8; 4096];
        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;
            debug!("Received {} bytes from {}", len, src);
            let src_ip = src.ip();
            if let Some(nas_secret) = self.nas_manager.find_secret(src_ip) {
                debug!("Found NAS secret for {}: {}", src_ip, nas_secret.secret);
                // TODO: Use secret for packet validation/decryption
            } else {
                error!("No NAS secret found for {}", src_ip);
                continue;
            }
            if let Some(packet) = RadiusPacket::parse(&buf[..len]) {
                debug!("Parsed RADIUS packet: {:?}", packet);
                // TODO: Handle authentication logic and send response
            } else {
                error!("Failed to parse RADIUS packet from {}", src);
            }
        }
    }
} 