use tokio::net::UdpSocket;
use tracing::{info, debug, error};
use std::net::IpAddr;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::auth::AuthServer;
use hmac::Mac;
use md5::Md5;
use digest::{Digest, CtOutput};
use sqlx::PgPool;

// Add these constants at the top of the file
const ATTR_USER_PASSWORD: u8 = 2;      // PAP
const ATTR_CHAP_PASSWORD: u8 = 3;      // CHAP
const ATTR_MS_CHAP_CHALLENGE: u8 = 11; // MS-CHAP
const ATTR_MS_CHAP_RESPONSE: u8 = 1;   // MS-CHAP
const ATTR_MS_CHAP2_RESPONSE: u8 = 25; // MS-CHAPv2
const ATTR_USER_NAME: u8 = 1;

const ATTR_REPLY_MESSAGE: u8 = 18;  // Reply-Message attribute type


#[derive(Debug, Clone)]
pub struct RadiusAttribute {
    pub typ: u8,
    pub value: Vec<u8>,
}

#[derive(Debug)]
pub enum AuthResult {
    Success,
    UserNotFound,
    InvalidPassword,
    AccountDisabled,
    DatabaseError(sqlx::Error),
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
    socket: UdpSocket,
    auth_server: Arc<AuthServer>,
}

impl RadiusAuthServer {
    pub async fn new(bind_addr: String, auth_server: Arc<AuthServer>) -> Result<Self, Box<dyn std::error::Error>> {
        debug!("Attempting to bind to address: {}", bind_addr);
        let socket = UdpSocket::bind(&bind_addr).await?;
        debug!("Successfully bound to {}", bind_addr);
        
        // Get the local address we're bound to
        let local_addr = socket.local_addr()?;
        info!("RADIUS Auth server listening on {}", local_addr);
        
        Ok(Self {
            socket,
            auth_server,
        })
    }

    fn detect_auth_method(&self, packet: &RadiusPacket) -> String {
        if packet.attributes.iter().any(|attr| attr.typ == ATTR_USER_PASSWORD) {
            "PAP".to_string()
        } else if packet.attributes.iter().any(|attr| attr.typ == ATTR_CHAP_PASSWORD) {
            "CHAP".to_string()
        } else if packet.attributes.iter().any(|attr| attr.typ == ATTR_MS_CHAP2_RESPONSE) {
            "MS-CHAPv2".to_string()
        } else if packet.attributes.iter().any(|attr| attr.typ == ATTR_MS_CHAP_RESPONSE) {
            "MS-CHAP".to_string()
        } else {
            "Unknown".to_string()
        }
    }


    async fn authenticate_user(&self, username: &str, password: Vec<u8>, authenticator: &[u8],
                               secret: &str
    ) -> Result<AuthResult, sqlx::Error> {
        let pool = self.auth_server.get_pool();
        
        // Query the user_identifiers table
        let result = sqlx::query!(
            r#"
            SELECT id, plain_password, is_enabled
            FROM user_identifiers 
            WHERE value = $1 AND identifier_type_id = 1
            "#,
            username
        )
        .fetch_optional(pool)
        .await?;

        match result {
            Some(record) => {
                debug!("User found: {} with ID {}", username, record.id);

                // First, check if an account is enabled - handle Option<bool>
                if !record.is_enabled {  // If is_enabled is just a bool
                    return Ok(AuthResult::AccountDisabled);
                }

                // Then check the password
                match &record.plain_password {
                    Some(stored_pass) => {
                        debug!("Found stored password for user: {}", username);

                        // Decode the RADIUS PAP password
                        let decoded_password = match decode_pap_password(password, authenticator, secret) {
                            Ok(decoded) => decoded,
                            Err(e) => {
                                debug!("Failed to decode PAP password: {}", e);
                                return Ok(AuthResult::InvalidPassword);
                            }
                        };

                        debug!("Comparing passwords for user {}: stored_length={}, decoded_length={}",
                        username, stored_pass.len(), decoded_password.len());

                        if *stored_pass == decoded_password {
                            debug!("Password match successful for user: {}", username);
                            Ok(AuthResult::Success)
                        } else {
                            debug!("Password mismatch for user: {}. Check PAP decoding", username);
                            Ok(AuthResult::InvalidPassword)
                        }
                    },
                    None => {
                        debug!("No password stored for user: {}", username);
                        Ok(AuthResult::InvalidPassword)
                    }
                }

            }
            None => {
                debug!("User not found: {}", username);
                Ok(AuthResult::UserNotFound)
            }
        }

    }

    // Add this function to decode PAP passwords
    fn create_access_reject(&self, request: &RadiusPacket, secret: &str, reason: &str) -> Vec<u8> {
        // Create the basic Access-Reject packet
        let mut response = RadiusPacket {
            code: 3, // Access-Reject
            identifier: request.identifier,
            length: 20, // Initial length with just header, will be updated
            authenticator: request.authenticator,
            attributes: vec![
                // Add Reply-Message attribute with the reason
                RadiusAttribute {
                    typ: ATTR_REPLY_MESSAGE,
                    value: reason.as_bytes().to_vec(),
                },
            ],
        };

        // Encode the packet to get the complete structure
        let mut encoded = response.encode();

        // Update the length field
        let length = encoded.len() as u16;
        encoded[2] = (length >> 8) as u8;
        encoded[3] = (length as u8);

        // Calculate Response Authenticator
        let mut hasher = md5::Md5::new();
        hasher.update(&encoded[0..4]); // Code+ID+Length
        hasher.update(&request.authenticator); // RequestAuth
        hasher.update(&encoded[20..]); // Attributes
        hasher.update(secret.as_bytes()); // Secret
        let response_auth = hasher.finalize();

        // Copy Response Authenticator into the packet
        encoded[4..20].copy_from_slice(&response_auth);

        encoded
    }


    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = vec![0u8; 4096];
        debug!("Starting RADIUS server loop");
        
        loop {
            debug!("Waiting for incoming packets...");
            match self.socket.recv_from(&mut buf).await {
                Ok((size, src)) => {
                    debug!("Received {} bytes from {}", size, src);
                    
                    let ip = src.ip();
                    debug!("Source IP: {}", ip);
                    
                    if let Some(secret) = self.auth_server.find_secret_for_ip(ip) {
                        debug!("Found secret for IP {}: {}", ip, secret);
                        
                        if let Some(packet) = RadiusPacket::parse(&buf[..size]) {
                            debug!("Parsed RADIUS packet: {:?}", packet);
                            
                            match packet.code {
                                1 => { // Access-Request
                                    let auth_method = self.detect_auth_method(&packet);
                                    debug!("Received Access-Request with {} authentication", auth_method);
                                    
                                    // Extract username and password for PAP
                                    let mut username = None;
                                    let mut password = None;
                                    
                                    for attr in &packet.attributes {
                                        match attr.typ {
                                            ATTR_USER_NAME => {
                                                username = Some(String::from_utf8_lossy(&attr.value).to_string());
                                            }
                                            ATTR_USER_PASSWORD => {
                                                password = Some(attr.value.clone()); // Changed this line - now storing raw bytes
                                            }

                                            _ => {}
                                        }
                                    }

                                    let response = if auth_method == "PAP" {
                                        if let (Some(username), Some(password)) = (username, password) {
                                            debug!("Authenticating user: {}", username);
                                            match self.authenticate_user(&username,
                                                                         password,  // encoded password
                                                                         &packet.authenticator,
                                                                         secret
                                            ).await {
                                                Ok(auth_result) => match auth_result {
                                                    AuthResult::Success => {
                                                        debug!("Authentication successful for user: {}", username);
                                                        self.create_access_accept(&packet, secret)
                                                    }
                                                    AuthResult::UserNotFound => {
                                                        debug!("User not found: {}", username);
                                                        self.create_access_reject(&packet, secret, "User not found")
                                                    }
                                                    AuthResult::InvalidPassword => {
                                                        debug!("Invalid password for user: {}", username);
                                                        self.create_access_reject(&packet, secret, "Invalid password")
                                                    }
                                                    AuthResult::AccountDisabled => {
                                                        debug!("Account disabled for user: {}", username);
                                                        self.create_access_reject(&packet, secret, "Account is disabled")
                                                    }
                                                    AuthResult::DatabaseError(e) => {
                                                        error!("Database error during authentication: {}", e);
                                                        self.create_access_reject(&packet, secret, "Internal server error")
                                                    }
                                                },

                                                Err(e) => {
                                                    error!("Database error during authentication: {}", e);
                                                    self.create_access_reject(&packet, secret, "Internal server error")
                                                }
                                            }
                                        } else {
                                            error!("Missing username or password in PAP request");
                                            self.create_access_reject(&packet, secret, "Missing username or password")
                                        }
                                    } else {
                                        debug!("Unsupported authentication method: {}", auth_method);
                                        self.create_access_reject(&packet, secret, "Unsupported authentication method")
                                    };

                                    debug!("Sending response");
                                    if let Err(e) = self.socket.send_to(&response, src).await {
                                        error!("Failed to send response: {}", e);
                                    }
                                }
                                4 => { // Accounting-Request
                                    debug!("Received Accounting-Request");
                                    // TODO: Implement accounting logic
                                    // For now, just send Accounting-Response
                                    let response = self.create_accounting_response(&packet);
                                    debug!("Sending Accounting-Response");
                                    if let Err(e) = self.socket.send_to(&response, src).await {
                                        error!("Failed to send response: {}", e);
                                    }
                                }
                                _ => {
                                    error!("Unsupported packet code: {}", packet.code);
                                }
                            }
                        } else {
                            error!("Failed to parse RADIUS packet from {}", src);
                        }
                    } else {
                        error!("No NAS secret found for {}", ip);
                        continue;
                    }
                }
                Err(e) => {
                    error!("Error receiving packet: {}", e);
                }
            }
        }
    }

    fn create_access_accept(&self, request: &RadiusPacket, secret: &str) -> Vec<u8> {
        // Create the basic Access-Accept packet
        let mut response = RadiusPacket {
            code: 2, // Access-Accept
            identifier: request.identifier,
            length: 20, // Initial length with just header, will be updated
            authenticator: request.authenticator,
            attributes: Vec::new(),
        };

        // Add Message-Authenticator if it was in the request
        if request.attributes.iter().any(|attr| attr.typ == 80) {
            response.attributes.push(RadiusAttribute {
                typ: 80,
                value: vec![0u8; 16], // Will be calculated later
            });
        }

        // Encode the packet to get the complete structure
        let mut encoded = response.encode();

        // Update the length field
        let length = encoded.len() as u16;
        encoded[2] = (length >> 8) as u8;
        encoded[3] = (length as u8);

        // Calculate Message-Authenticator if present
        if let Some(msg_auth_pos) = encoded.windows(2).position(|w| w[0] == 80) {
            // Create HMAC-MD5 of the packet with zeroed Message-Authenticator
            type HmacMd5 = hmac::Hmac<md5::Md5>;
            let mut mac = HmacMd5::new_from_slice(secret.as_bytes())
                .expect("HMAC can take key of any size");

            // Create temporary buffer with zeroed Message-Authenticator
            let mut temp_packet = encoded.clone();
            for i in 0..16 {
                temp_packet[msg_auth_pos + 2 + i] = 0;
            }

            mac.update(&temp_packet);
            let result = mac.finalize();
            let message_auth = result.into_bytes();

            // Copy the calculated Message-Authenticator into the packet
            encoded[msg_auth_pos + 2..msg_auth_pos + 18].copy_from_slice(&message_auth);
        }

        // Calculate Response Authenticator
        // Response Auth = MD5(Code+ID+Length+RequestAuth+Attributes+Secret)
        let mut hasher = md5::Md5::new();
        hasher.update(&encoded[0..4]); // Code+ID+Length
        hasher.update(&request.authenticator); // RequestAuth
        hasher.update(&encoded[20..]); // Attributes
        hasher.update(secret.as_bytes()); // Secret
        let response_auth = hasher.finalize();

        // Copy Response Authenticator into the packet
        encoded[4..20].copy_from_slice(&response_auth);

        encoded
    }

    fn create_accounting_response(&self, request: &RadiusPacket) -> Vec<u8> {
        let mut response = RadiusPacket {
            code: 5, // Accounting-Response
            identifier: request.identifier,
            length: 0, // Will be set after adding attributes
            authenticator: [0u8; 16], // Will be set after adding attributes
            attributes: Vec::new(),
        };

        // Encode the response
        let mut encoded = response.encode();

        // Update length field
        let length = encoded.len() as u16;
        encoded[2] = (length >> 8) as u8;
        encoded[3] = length as u8;

        encoded
    }
}


fn decode_pap_password(encrypted: Vec<u8>, authenticator: &[u8], secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try to extract password from quotes first

    debug!("Decrypting PAP password: encrypted_len={}, authenticator={:?}",
           encrypted.len(), authenticator);

    let mut decrypted = Vec::with_capacity(encrypted.len());
    let mut last_block = authenticator;

    // Process the password in 16 byte blocks as per RFC 2865
    for chunk in encrypted.chunks(16) {
        // Calculate the MD5 hash of the secret concatenated with the last block
        let mut hasher = Md5::new();
        hasher.update(secret.as_bytes());
        hasher.update(last_block);
        let hash = hasher.finalize();

        // XOR the hash with the encrypted chunk to get the plaintext
        for (i, &byte) in chunk.iter().enumerate() {
            decrypted.push(byte ^ hash[i]);
        }

        // The encrypted chunk becomes the "last block" for the next iteration
        last_block = chunk;
    }

    // Remove padding (null bytes from the end)
    while let Some(&0) = decrypted.last() {
        decrypted.pop();
    }

    // Convert to UTF-8 string
    match String::from_utf8(decrypted) {
        Ok(s) => {
            debug!("Successfully decrypted PAP password, length: {}", s.len());
            Ok(s)
        },
        Err(e) => {
            error!("Failed to convert decrypted password to UTF-8: {}", e);
            Err(Box::new(e))
        }
    }

}