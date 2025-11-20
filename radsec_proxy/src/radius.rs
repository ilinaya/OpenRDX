use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{debug, error};

#[derive(Debug)]
pub struct RadiusPacket {
    pub code: u8,
    pub identifier: u8,
    pub length: u16,
    pub authenticator: [u8; 16],
    pub attributes: Vec<u8>,
}

impl RadiusPacket {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }

        let length = u16::from_be_bytes([data[2], data[3]]);
        if data.len() != length as usize {
            return None;
        }

        let mut authenticator = [0u8; 16];
        authenticator.copy_from_slice(&data[4..20]);

        Some(RadiusPacket {
            code: data[0],
            identifier: data[1],
            length,
            authenticator,
            attributes: data[20..].to_vec(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.length as usize);
        result.push(self.code);
        result.push(self.identifier);
        result.extend_from_slice(&self.length.to_be_bytes());
        result.extend_from_slice(&self.authenticator);
        result.extend_from_slice(&self.attributes);
        result
    }
}

pub async fn forward_packet(
    packet: &RadiusPacket,
    server_addr: &str,
    _secret: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr: SocketAddr = server_addr.parse()?;
    
    // Send the packet
    socket.send_to(&packet.to_bytes(), server_addr).await?;
    
    // Receive response
    let mut buf = vec![0u8; 4096];
    let (n, _) = socket.recv_from(&mut buf).await?;
    
    Ok(buf[..n].to_vec())
} 