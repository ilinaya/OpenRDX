use crate::warn;
use tokio::net::UdpSocket;
use tracing::{info, debug, error};
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use crate::auth::AuthServer;
use hmac::{Hmac, Mac};
use md5::{Md5};

use digest::{Digest, KeyInit};
use rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use rustls_pemfile::{certs, pkcs8_private_keys};
use des::cipher::{BlockEncrypt};
use generic_array::GenericArray;
type HmacMd5 = Hmac<Md5>; // alias for clarity

const ATTR_USER_PASSWORD: u8 = 2;      // PAP
const ATTR_CHAP_PASSWORD: u8 = 3;      // CHAP
const VENDOR_MICROSOFT: u32 = 311;       // Microsoft's Vendor-ID

const VENDOR_ATTR_MS_CHAP_CHALLENGE: u8 = 11;  // Microsoft's MS-CHAP-Challenge
const VENDOR_ATTR_MS_CHAP_RESPONSE: u8 = 1;    // Microsoft's MS-CHAP-Response

const ATTR_VENDOR_SPECIFIC: u8 = 26;   // VSA attribute type

const ATTR_MS_CHAP2_RESPONSE: u8 = 25; // MS-CHAPv2

const VENDOR_ATTR_MS_CHAP2_RESPONSE: u8 = 25;    // Microsoft's MS-CHAPv2-Response
const VENDOR_ATTR_MS_CHAP2_CHALLENGE: u8 = 11;   // Microsoft's MS-CHAPv2-Challenge
const VENDOR_ATTR_MS_CHAP2_SUCCESS: u8 = 26;     // Microsoft's MS-CHAP2-Success
const VENDOR_ATTR_MS_MPPE_ENCRYPTION_POLICY: u8 = 7;  // Microsoft's MS-MPPE-Encryption-Policy
const VENDOR_ATTR_MS_MPPE_ENCRYPTION_TYPES: u8 = 8;   // Microsoft's MS-MPPE-Encryption-Types
const VENDOR_ATTR_MS_MPPE_SEND_KEY: u8 = 16;          // Microsoft's MS-MPPE-Send-Key
const VENDOR_ATTR_MS_MPPE_RECV_KEY: u8 = 17;          // Microsoft's MS-MPPE-Recv-Key

const ATTR_USER_NAME: u8 = 1;
const ATTR_NAS_IDENTIFIER: u8 = 32;  // NAS-Identifier attribute type

const ATTR_REPLY_MESSAGE: u8 = 18;  // Reply-Message attribute type

// EAP-related constants
const ATTR_EAP_MESSAGE: u8 = 79;      // EAP-Message attribute
const ATTR_MESSAGE_AUTHENTICATOR: u8 = 80;  // Message-Authenticator attribute

// EAP method types
const EAP_TYPE_TLS: u8 = 13;          // EAP-TLS
const EAP_TYPE_TTLS: u8 = 21;         // EAP-TTLS
const EAP_TYPE_PEAP: u8 = 25;         // EAP-PEAP
const EAP_TYPE_SIM: u8 = 18;          // EAP-SIM
const EAP_TYPE_AKA: u8 = 23;          // EAP-AKA
const EAP_TYPE_AKA_PRIME: u8 = 50;    // EAP-AKA'

// EAP-SIM/AKA subtypes
const EAP_SIM_START: u8 = 10;
const EAP_SIM_CHALLENGE: u8 = 11;
const EAP_SIM_NOTIFICATION: u8 = 12;
const EAP_SIM_REAUTHENTICATION: u8 = 13;
const EAP_SIM_CLIENT_ERROR: u8 = 14;

// EAP-AKA specific subtypes
const EAP_AKA_CHALLENGE: u8 = 1;
const EAP_AKA_AUTHENTICATION_REJECT: u8 = 2;
const EAP_AKA_SYNCHRONIZATION_FAILURE: u8 = 4;
const EAP_AKA_IDENTITY: u8 = 5;
const EAP_AKA_NOTIFICATION: u8 = 12;
const EAP_AKA_REAUTHENTICATION: u8 = 13;
const EAP_AKA_CLIENT_ERROR: u8 = 14;

// EAP codes
const EAP_REQUEST: u8 = 1;
const EAP_RESPONSE: u8 = 2;
const EAP_SUCCESS: u8 = 3;
const EAP_FAILURE: u8 = 4;

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

pub struct Mschapv2Result {
    pub result: AuthResult,
    pub authenticator_response: Option<Vec<u8>>,
    pub password_hash: Option<Vec<u8>>,
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

        // Check if the attribute length exceeds the maximum allowed size (255 bytes)
        let attr_len = self.value.len() + 2;
        if attr_len > 255 {
            error!("Attribute type {} length {} exceeds maximum allowed size of 255 bytes, truncating", 
                  self.typ, attr_len);
            let truncated_len = 253; // 255 - 2 bytes for type and length
            out.push(255); // Maximum length
            out.extend_from_slice(&self.value[0..truncated_len]);

            // Log the truncated attribute for debugging
            debug!("Truncated attribute type {}: original_length={}, truncated_length={}",
                  self.typ, attr_len, 255);
        } else {
            out.push(attr_len as u8);
            out.extend_from_slice(&self.value);

            // Log the attribute for debugging
            debug!("Encoded attribute type {}: length={}", self.typ, attr_len);
        }

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
        // Calculate total length first
        let mut total_length = 20; // Header size
        let mut attr_lengths = Vec::new();

        for attr in &self.attributes {
            let attr_len = attr.encode().len();
            total_length += attr_len;
            attr_lengths.push(attr_len);
        }

        debug!("Encoding RADIUS packet: code={}, identifier={}, attributes={}, calculated_length={}",
               self.code, self.identifier, self.attributes.len(), total_length);

        // Ensure length doesn't exceed maximum RADIUS packet size (4096 bytes)
        if total_length > 4096 {
            error!("Packet length {} exceeds maximum RADIUS packet size of 4096 bytes", total_length);
            total_length = 4096;
        }

        // Start with a capacity based on the calculated length
        let mut out = Vec::with_capacity(total_length);
        out.push(self.code);
        out.push(self.identifier);

        // Add length field
        out.push((total_length >> 8) as u8);
        out.push(total_length as u8);
        out.extend_from_slice(&self.authenticator);

        // Add attributes
        let mut skipped_attrs = 0;
        for (i, attr) in self.attributes.iter().enumerate() {
            let attr_bytes = attr.encode();
            if out.len() + attr_bytes.len() <= 4096 {
                debug!("Adding attribute {}: type={}, length={}", i, attr.typ, attr_bytes.len());
                out.extend_from_slice(&attr_bytes);
            } else {
                error!("Skipping attribute {} (type={}, length={}) that would exceed maximum packet size",
                       i, attr.typ, attr_bytes.len());
                skipped_attrs += 1;
                break;
            }
        }

        if skipped_attrs > 0 {
            error!("Skipped {} attributes due to packet size limitations", skipped_attrs);
        }

        // Verify final length matches what we set
        let final_length = out.len() as u16;
        if final_length != total_length as u16 {
            error!("Final packet length {} doesn't match calculated length {}", final_length, total_length);
            out[2] = (final_length >> 8) as u8;
            out[3] = final_length as u8;
        }

        debug!("Final encoded packet: length={}, attributes={}", out.len(), self.attributes.len() - skipped_attrs);
        out
    }
}

pub struct RadiusAuthServer {
    socket: UdpSocket,
    auth_server: Arc<AuthServer>,
    // Add connection tracking
    connections: Arc<tokio::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>>,
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
            connections: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        })
    }

    fn detect_auth_method(&self, packet: &RadiusPacket) -> String {
        // Check for EAP first
        if packet.attributes.iter().any(|attr| attr.typ == ATTR_EAP_MESSAGE) {
            return "EAP".to_string();
        }

        // Check for MS-CHAP and MS-CHAPv2 in Vendor-Specific Attributes
        // Check for MS-CHAP and MS-CHAPv2 in Vendor-Specific Attributes
        let has_ms_auth = packet.attributes.iter().any(|attr| {
            if attr.typ == ATTR_VENDOR_SPECIFIC && attr.value.len() >= 4 {
                let vendor_id = u32::from_be_bytes([
                    attr.value[0], attr.value[1],
                    attr.value[2], attr.value[3]
                ]);
                if vendor_id == VENDOR_MICROSOFT && attr.value.len() >= 6 {
                    let vendor_type = attr.value[4];
                    return vendor_type == VENDOR_ATTR_MS_CHAP_RESPONSE ||
                        vendor_type == VENDOR_ATTR_MS_CHAP2_RESPONSE;
                }
            }
            false
        });

        if has_ms_auth {
            // Now determine if it's MS-CHAP or MS-CHAPv2
            let is_v2 = packet.attributes.iter().any(|attr| {
                if attr.typ == ATTR_VENDOR_SPECIFIC && attr.value.len() >= 4 {
                    let vendor_id = u32::from_be_bytes([
                        attr.value[0], attr.value[1],
                        attr.value[2], attr.value[3]
                    ]);
                    if vendor_id == VENDOR_MICROSOFT && attr.value.len() >= 6 {
                        return attr.value[4] == VENDOR_ATTR_MS_CHAP2_RESPONSE;
                    }
                }
                false
            });

            if is_v2 {
                return "MS-CHAPv2".to_string();
            } else {
                return "MS-CHAP".to_string();
            }
        }


        if packet.attributes.iter().any(|attr| attr.typ == ATTR_USER_PASSWORD) {
            "PAP".to_string()
        } else if packet.attributes.iter().any(|attr| attr.typ == ATTR_CHAP_PASSWORD) {
            "CHAP".to_string()
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

    fn create_access_reject(&self, request: &RadiusPacket, secret: &str, reason: &str) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        use md5::Md5;
        use digest::KeyInit;

        // Ensure Reply-Message is not too long
        let mut reply_msg = reason.as_bytes().to_vec();
        if reply_msg.len() > 253 {
            reply_msg.truncate(253);
        }

        let mut attributes = vec![
            RadiusAttribute {
                typ: ATTR_REPLY_MESSAGE,
                value: reply_msg,
            },
        ];

        let has_msg_auth = request.attributes.iter().any(|attr| attr.typ == ATTR_MESSAGE_AUTHENTICATOR);
        if has_msg_auth {
            attributes.push(RadiusAttribute {
                typ: ATTR_MESSAGE_AUTHENTICATOR,
                value: vec![0u8; 16],
            });
        }

        let mut response = RadiusPacket {
            code: 3, // Access-Reject
            identifier: request.identifier,
            length: 20,
            authenticator: request.authenticator,
            attributes,
        };

        let mut encoded = response.encode();

        if has_msg_auth {
            if let Some(pos) = encoded.windows(2).position(|w| w[0] == ATTR_MESSAGE_AUTHENTICATOR && w[1] == 18) {
                let mut temp = encoded.clone();
                for i in 0..16 {
                    temp[pos + 2 + i] = 0;
                }

                let mut mac = <Hmac<Md5> as KeyInit>::new_from_slice(secret.as_bytes()).unwrap();
                mac.update(&encoded); // MsgAuth=0
                let msg_auth = mac.finalize().into_bytes();

                // Set Message-Authenticator in encoded
                encoded[pos + 2..pos + 18].copy_from_slice(&msg_auth);

                // --- (D) --- Now recalculate Response Authenticator (MD5) with correct buffer
                let mut md5 = Md5::new();
                md5.update(&encoded[0..4]);
                md5.update(&request.authenticator);
                md5.update(&encoded[20..]);
                md5.update(secret.as_bytes());
                let response_auth = md5.finalize();
                encoded[4..20].copy_from_slice(&response_auth);


                debug!("✅ Reject Response Authenticator: {:02X?}", response_auth);
                debug!("✅ Reject Message-Authenticator: {:02X?}", msg_auth);
            } else {
                error!("❌ Could not locate Message-Authenticator attribute in Access-Reject");
            }
        } else {
            // No Msg-Auth: Still need to calculate Response Authenticator
            let mut md5 = Md5::new();
            md5.update(&encoded[0..4]);
            md5.update(&request.authenticator);
            md5.update(&encoded[20..]);
            md5.update(secret.as_bytes());
            let response_auth = md5.finalize();
            encoded[4..20].copy_from_slice(&response_auth);

            debug!("✅ Reject Response Authenticator (no MsgAuth): {:02X?}", response_auth);
        }

        debug!("✅ Final Access-Reject packet: {:02X?}", encoded);
        encoded
    }

    async fn handle_packet(&self, data: &[u8], src: std::net::SocketAddr, secret: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Parse the packet
        let packet = match RadiusPacket::parse(data) {
            Some(p) => p,
            None => return Err("Invalid packet format".into()),
        };

        // Extract NAS-Identifier from packet
        let nas_identifier = packet.attributes.iter()
            .find(|attr| attr.typ == ATTR_NAS_IDENTIFIER)
            .and_then(|attr| String::from_utf8(attr.value.clone()).ok());

        if let Some(ref nas_id) = nas_identifier {
            debug!("Found NAS-Identifier in packet: {}", nas_id);
            
            // Try to find NAS device by identifier
            if let Some(nas_device) = self.auth_server.find_nas_device_by_identifier(nas_id) {
                debug!("Matched NAS device: {} (ID: {})", nas_device.name, nas_device.id);
            } else {
                warn!("No NAS device found for identifier: {}", nas_id);
            }
        } else {
            debug!("No NAS-Identifier found in packet, falling back to IP-based matching");
        }

        // Check for Message-Authenticator
        let mut has_msg_auth = false;
        let mut msg_auth_value = None;

        for attr in &packet.attributes {
            if attr.typ == ATTR_MESSAGE_AUTHENTICATOR {
                has_msg_auth = true;
                msg_auth_value = Some(attr.value.clone());

                if let Some(ref value) = msg_auth_value {
                    let hex = value.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    debug!("Message-Authenticator from client = 0x{}", hex);
                }

                break;
            }
        }

        // Validate Message-Authenticator if present
        if has_msg_auth {
            if let Some(auth_value) = &msg_auth_value {
                debug!("Validating Message-Authenticator for packet: code={}, identifier={}", 
                       packet.code, packet.identifier);

                let is_valid = self.validate_message_authenticator(&packet, secret, auth_value);
                if !is_valid {
                    error!("Invalid Message-Authenticator in packet from {}", src);
                    return Err("Invalid Message-Authenticator".into());
                } else {
                    debug!("Message-Authenticator validation successful");
                }
            }
        }

        // Process the packet based on its code
        match packet.code {
            1 => { // Access-Request
                Ok(self.handle_access_request(&packet, secret, msg_auth_value).await)
            }
            4 => { // Accounting-Request
                Ok(self.create_accounting_response(&packet))
            }
            _ => {
                debug!("Unsupported packet code: {}", packet.code);
                Err("Unsupported packet code".into())
            }
        }
    }

    fn validate_message_authenticator(&self, packet: &RadiusPacket, secret: &str, received_auth: &[u8]) -> bool {
        use hmac::{Hmac, Mac};
        use md5::Md5;

        let mut encoded = packet.encode();

        // Manually walk the encoded AVP section and zero out the Message-Authenticator
        let mut pos = 20; // RADIUS header is 20 bytes

        while pos + 2 <= encoded.len() {
            let attr_type = encoded[pos];
            let attr_len = encoded[pos + 1] as usize;

            if attr_len < 2 || pos + attr_len > encoded.len() {
                // Invalid attribute, bail
                break;
            }

            if attr_type == ATTR_MESSAGE_AUTHENTICATOR && attr_len == 18 {
                // Zero out the 16-byte Message-Authenticator value (starts at pos + 2)
                for i in 0..16 {
                    encoded[pos + 2 + i] = 0;
                }
                break; // done
            }

            pos += attr_len;
        }

        // Calculate expected Message-Authenticator (RFC 2869)
        let mut mac = <HmacMd5 as KeyInit>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");

        mac.update(&encoded);
        let expected_auth = mac.finalize().into_bytes();

        if received_auth != expected_auth.as_slice() {
            debug!("Message-Authenticator validation failed");
            debug!("Expected: {:02x?}", expected_auth);
            debug!("Received: {:02x?}", received_auth);
            return false;
        }

        true
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting RADIUS server loop");

        loop {
            // Create a new buffer for each request to prevent data corruption
            let mut buf = vec![0u8; 4096];
            debug!("Waiting for incoming packets...");
            match self.socket.recv_from(&mut buf).await {
                Ok((size, src)) => {
                    debug!("Received {} bytes from {}", size, src);

                    let ip = src.ip();
                    debug!("Source IP: {}", ip);

                    // Parse packet to extract NAS-Identifier
                    let nas_identifier = if let Some(packet) = RadiusPacket::parse(&buf[..size]) {
                        packet.attributes.iter()
                            .find(|attr| attr.typ == ATTR_NAS_IDENTIFIER)
                            .and_then(|attr| String::from_utf8(attr.value.clone()).ok())
                    } else {
                        None
                    };

                    // Try to find secret by NAS-Identifier first, then fall back to IP
                    let secret = if let Some(ref nas_id) = nas_identifier {
                        debug!("Looking up secret by NAS-Identifier: {}", nas_id);
                        // For now, still use IP-based secret lookup, but we could extend this
                        // to store secrets per NAS device in the future
                        self.auth_server.find_secret_for_ip(ip)
                    } else {
                        debug!("No NAS-Identifier found, using IP-based lookup");
                        self.auth_server.find_secret_for_ip(ip)
                    };

                    if let Some(secret) = secret {
                        debug!("Found secret for IP {}: {}", ip, secret);

                        // Create a copy of the received data to ensure it's not modified by subsequent requests
                        let request_data = buf[..size].to_vec();

                        match self.handle_packet(&request_data, src, secret).await {
                            Ok(response) => {
                                debug!("Response packet size: {} bytes", response.len());
                                debug!("Response packet: {:?}", response);

                                // Verify the response packet has a valid length
                                if response.len() >= 20 {
                                    let length = u16::from_be_bytes([response[2], response[3]]);
                                    debug!("Response packet length field: {}", length);

                                    if length as usize != response.len() {
                                        error!("Response packet length field ({}) doesn't match actual length ({})", length, response.len());
                                    }
                                } else {
                                    error!("Response packet is too short: {} bytes", response.len());
                                }

                                if let Err(e) = self.socket.send_to(&response, src).await {
                                    error!("Failed to send response: {}", e);
                                } else {
                                    debug!("Successfully sent response to {}", src);
                                }
                            }
                            Err(e) => {
                                error!("Error handling packet: {}", e);
                                // Optionally send Access-Reject for certain errors
                                if let Some(packet) = RadiusPacket::parse(&request_data) {
                                    let reject = self.create_access_reject(&packet, secret, &format!("Error: {}", e));
                                    debug!("Reject packet size: {} bytes", reject.len());
                                    debug!("Reject packet: {:?}", reject);

                                    if let Err(e) = self.socket.send_to(&reject, src).await {
                                        error!("Failed to send reject: {}", e);
                                    } else {
                                        debug!("Successfully sent reject to {}", src);
                                    }
                                }
                            }
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

    async fn handle_access_request(&self, packet: &RadiusPacket, secret: &str, msg_auth_value: Option<Vec<u8>>) -> Vec<u8> {
        let auth_method = self.detect_auth_method(packet);
        debug!("Handling Access-Request with {} authentication", auth_method);

        let mut username = None;
        let mut password = None;
        let mut chap_id = None;
        let mut chap_response = None;

        let mut mschap_challenge = None;
        let mut mschap_response = None;

        let mut mschap2_peer_challenge = None;
        let mut mschap2_nt_response = None;


        // Extract all relevant attributes
        for attr in &packet.attributes {
            match attr.typ {
                ATTR_USER_NAME => {
                    username = Some(String::from_utf8_lossy(&attr.value).to_string());
                }
                ATTR_USER_PASSWORD => {
                    password = Some(attr.value.clone());
                }
                ATTR_CHAP_PASSWORD => {
                    if attr.value.len() >= 1 {
                        chap_id = Some(attr.value[0]);
                        chap_response = Some(attr.value[1..].to_vec());
                    }
                }
                // In your attribute extraction loop:
                ATTR_VENDOR_SPECIFIC => {
                    if attr.value.len() < 4 {
                        continue;
                    }
                    // Extract vendor ID (first 4 bytes)
                    let vendor_id = u32::from_be_bytes([
                        attr.value[0], attr.value[1],
                        attr.value[2], attr.value[3]
                    ]);

                    if vendor_id == VENDOR_MICROSOFT {
                        // Get vendor type and length
                        if attr.value.len() < 6 {
                            continue;
                        }
                        let vendor_type = attr.value[4];
                        //let vendor_length = attr.value[5] as usize;
                        let vendor_data = &attr.value[6..];

                        match vendor_type {
                            // Check MS-CHAPv2-Challenge first (before MS-CHAP-Challenge) since they both use attribute 11
                            VENDOR_ATTR_MS_CHAP2_CHALLENGE => {
                                debug!("Found MS-CHAPv2-Challenge in VSA");
                                // MS-CHAPv2-Challenge is 16 bytes
                                if vendor_data.len() >= 16 {
                                    mschap_challenge = Some(vendor_data[0..16].to_vec());
                                    debug!("Extracted MS-CHAPv2-Challenge: {} bytes", vendor_data[0..16].len());
                                } else {
                                    warn!("MS-CHAPv2-Challenge VSA too short: {} bytes (expected at least 16)", vendor_data.len());
                                }
                            }
                            VENDOR_ATTR_MS_CHAP2_RESPONSE => {
                                debug!("Found MS-CHAPv2-Response in VSA, vendor_data length: {}", vendor_data.len());
                                if vendor_data.len() >= 50 {
                                    // MS-CHAPv2-Response format:
                                    // Byte 0: Identifier
                                    // Byte 1: Flags
                                    // Bytes 2-17: Peer-Challenge (16 bytes)
                                    // Bytes 18-25: Reserved (8 bytes)
                                    // Bytes 26-49: NT-Response (24 bytes)
                                    mschap2_peer_challenge = Some(vendor_data[2..18].to_vec());
                                    mschap2_nt_response = Some(vendor_data[26..50].to_vec());
                                    debug!("Extracted peer_challenge: {} bytes, nt_response: {} bytes", 
                                           vendor_data[2..18].len(), vendor_data[26..50].len());
                                } else {
                                    warn!("MS-CHAPv2-Response VSA too short: {} bytes (expected at least 50)", vendor_data.len());
                                }
                            }
                            VENDOR_ATTR_MS_CHAP_CHALLENGE => {
                                debug!("Found MS-CHAP-Challenge in VSA (v1, not v2)");
                                // For MS-CHAPv2, we should NOT use MS-CHAP-Challenge (v1)
                                // We'll use the RADIUS authenticator instead
                                // Only set this if we're doing MS-CHAP v1 (not v2)
                                // For now, we'll ignore it for MS-CHAPv2 and use authenticator
                            }
                            VENDOR_ATTR_MS_CHAP_RESPONSE => {
                                debug!("Found MS-CHAP-Response in VSA");
                                if vendor_data.len() >= 49 {
                                    mschap_response = Some(vendor_data[26..50].to_vec());
                                }
                            }

                            _ => {
                                debug!("Unknown Microsoft VSA type: {}", vendor_type);
                            }
                        }
                    }
                }

                //
                _ => {}
            }
        }

        match auth_method.as_str() {
            "PAP" => {
                if let (Some(username), Some(password)) = (username, password) {
                    match self.authenticate_user(&username, password, &packet.authenticator, secret).await {
                        Ok(AuthResult::Success) => self.create_access_accept(packet, secret),
                        Ok(AuthResult::UserNotFound) => self.create_access_reject(packet, secret, "User not found"),
                        Ok(AuthResult::InvalidPassword) => self.create_access_reject(packet, secret, "Invalid password"),
                        Ok(AuthResult::AccountDisabled) => self.create_access_reject(packet, secret, "Account is disabled"),
                        Ok(AuthResult::DatabaseError(_)) => self.create_access_reject(packet, secret, "Internal server error"),
                        Err(_) => self.create_access_reject(packet, secret, "Internal server error"),
                    }
                } else {
                    self.create_access_reject(packet, secret, "Missing username or password")
                }
            }
            "CHAP" => {

                if let (Some(username), Some(chap_id), Some(chap_response)) = (username, chap_id, chap_response) {
                    match self.authenticate_chap(&username, chap_id, &chap_response, &packet.authenticator, secret).await {
                        Ok(AuthResult::Success) => self.create_access_accept(packet, secret),
                        Ok(AuthResult::UserNotFound) => self.create_access_reject(packet, secret, "User not found"),
                        Ok(AuthResult::InvalidPassword) => self.create_access_reject(packet, secret, "Invalid CHAP response"),
                        Ok(AuthResult::AccountDisabled) => self.create_access_reject(packet, secret, "Account is disabled"),
                        Ok(AuthResult::DatabaseError(_)) => self.create_access_reject(packet, secret, "Internal server error"),
                        Err(_) => self.create_access_reject(packet, secret, "Internal server error"),
                    }
                } else {
                    self.create_access_reject(packet, secret, "Invalid CHAP request")
                }
            }
            "MS-CHAP" => {
                debug!("MS-CHAP authentication details:");
                if let Some(ref challenge) = mschap_challenge {
                    debug!("Challenge length: {}", challenge.len());
                }
                if let Some(ref response) = mschap_response {
                    debug!("Response length: {}", response.len());
                }

                if let (Some(username), Some(challenge), Some(response)) = (username, mschap_challenge, mschap_response) {
                    match self.authenticate_mschap(&username, &challenge, &response, &packet.authenticator, secret).await {
                        Ok(AuthResult::Success) => self.create_access_accept(packet, secret),
                        Ok(AuthResult::UserNotFound) => self.create_access_reject(packet, secret, "User not found"),
                        Ok(AuthResult::InvalidPassword) => self.create_access_reject(packet, secret, "Invalid MS-CHAP response"),
                        Ok(AuthResult::AccountDisabled) => self.create_access_reject(packet, secret, "Account is disabled"),
                        Ok(AuthResult::DatabaseError(_)) => self.create_access_reject(packet, secret, "Internal server error"),
                        Err(_) => self.create_access_reject(packet, secret, "Internal server error"),
                    }
                } else {
                    self.create_access_reject(packet, secret, "Invalid MS-CHAP request")
                }
            }
            "MS-CHAPv2" => {
                debug!("Processing MS-CHAPv2 authentication");
                debug!("Username: {:?}, Peer-Challenge: {:?}, NT-Response: {:?}, Auth-Challenge: {:?}",
                       username.as_ref(), 
                       mschap2_peer_challenge.as_ref().map(|c| format!("{} bytes", c.len())),
                       mschap2_nt_response.as_ref().map(|r| format!("{} bytes", r.len())),
                       mschap_challenge.as_ref().map(|c| format!("{} bytes", c.len())));

                // Validate required fields
                if username.is_none() {
                    return self.create_access_reject(packet, secret, "MS-CHAPv2: Missing username");
                }
                if mschap2_peer_challenge.is_none() {
                    return self.create_access_reject(packet, secret, "MS-CHAPv2: Missing peer-challenge in MS-CHAPv2-Response");
                }
                if mschap2_nt_response.is_none() {
                    return self.create_access_reject(packet, secret, "MS-CHAPv2: Missing NT-Response in MS-CHAPv2-Response");
                }
                
                // For MS-CHAPv2, the authenticator challenge should come from:
                // 1. MS-CHAPv2-Challenge attribute (if present in Access-Request, which is unusual)
                // 2. RADIUS authenticator from the Access-Request (most common for first request)
                // 3. RADIUS authenticator from a previous Access-Challenge (for subsequent requests)
                // For the first Access-Request, we use the RADIUS authenticator
                let auth_challenge = if let Some(challenge) = mschap_challenge {
                    if challenge.len() != 16 {
                        return self.create_access_reject(packet, secret, 
                            &format!("MS-CHAPv2: Invalid challenge length: {} bytes (expected 16)", challenge.len()));
                    }
                    debug!("MS-CHAPv2: Using MS-CHAPv2-Challenge attribute as authenticator challenge: {:02x?}", challenge);
                    challenge
                } else {
                    debug!("MS-CHAPv2: No MS-CHAPv2-Challenge attribute found, using RADIUS authenticator as challenge: {:02x?}", packet.authenticator);
                    packet.authenticator.to_vec()
                };

                // Validate peer challenge length
                let peer_challenge = mschap2_peer_challenge.unwrap();
                if peer_challenge.len() != 16 {
                    return self.create_access_reject(packet, secret, 
                        &format!("MS-CHAPv2: Invalid peer-challenge length: {} bytes (expected 16)", peer_challenge.len()));
                }

                // Validate NT-Response length
                let nt_response = mschap2_nt_response.unwrap();
                if nt_response.len() < 24 {
                    return self.create_access_reject(packet, secret, 
                        &format!("MS-CHAPv2: Invalid NT-Response length: {} bytes (expected at least 24)", nt_response.len()));
                }

                // Extract MS-CHAPv2 identifier from the VSA
                let ms_chap_v2_ident = packet.attributes.iter()
                    .find_map(|attr| {
                        if attr.typ == ATTR_VENDOR_SPECIFIC && attr.value.len() >= 6 {
                            let vendor_id = u32::from_be_bytes([attr.value[0], attr.value[1], attr.value[2], attr.value[3]]);
                            let vendor_type = attr.value[4];
                            if vendor_id == VENDOR_MICROSOFT && vendor_type == VENDOR_ATTR_MS_CHAP2_RESPONSE {
                                if attr.value.len() >= 7 {
                                    Some(attr.value[6]) // <-- This is the identifier
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0); // fallback

                match self.authenticate_mschap2(&username.clone().unwrap(), &peer_challenge, &nt_response, &auth_challenge, secret).await {
                    Ok(mschapv2_result) => match mschapv2_result.result {
                        AuthResult::Success => {
                            if let Some(auth_resp) = mschapv2_result.authenticator_response {
                                debug!("MS-CHAPv2 authentication successful for user: {}", username.unwrap());
                                self.create_access_accept_mschapv2(
                                    packet,
                                    secret,
                                    ms_chap_v2_ident,
                                    &auth_resp,
                                    mschapv2_result.password_hash.as_deref().unwrap_or(&[]),
                                    &nt_response)
                            } else {
                                error!("MS-CHAPv2: Authentication succeeded but authenticator response is missing");
                                self.create_access_reject(packet, secret, "MS-CHAPv2: Internal error - authenticator response generation failed")
                            }
                        }
                        AuthResult::UserNotFound => {
                            debug!("MS-CHAPv2: User not found: {}", username.clone().unwrap());
                            self.create_access_reject(packet, secret, &format!("MS-CHAPv2: User '{}' not found", username.unwrap()))
                        }
                        AuthResult::InvalidPassword => {
                            debug!("MS-CHAPv2: Password validation failed for user: {}", username.unwrap());
                            self.create_access_reject(packet, secret, "MS-CHAPv2: Password incorrect or NT-Response validation failed")
                        }
                        AuthResult::AccountDisabled => {
                            debug!("MS-CHAPv2: Account disabled for user: {}", username.clone().unwrap());
                            self.create_access_reject(packet, secret, &format!("MS-CHAPv2: Account for user '{}' is disabled", username.unwrap()))
                        }
                        AuthResult::DatabaseError(e) => {
                            error!("MS-CHAPv2: Database error: {:?}", e);
                            self.create_access_reject(packet, secret, "MS-CHAPv2: Database error during authentication")
                        }
                    },
                    Err(e) => {
                        error!("MS-CHAPv2: Authentication error: {:?}", e);
                        self.create_access_reject(packet, secret, &format!("MS-CHAPv2: Authentication error: {}", e))
                    }
                }
            }
            _ => self.create_access_reject(packet, secret, "Unsupported authentication method"),
        }
    }

    async fn authenticate_chap(&self, username: &str, chap_id: u8, chap_response: &[u8], authenticator: &[u8], secret: &str) -> Result<AuthResult, sqlx::Error> {
        let pool = self.auth_server.get_pool();

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
                if !record.is_enabled {
                    return Ok(AuthResult::AccountDisabled);
                }

                if let Some(stored_pass) = record.plain_password {
                    // Calculate expected CHAP response
                    let challenge = authenticator;

                    // Calculate expected CHAP response
                    let mut hasher = md5::Md5::new();
                    hasher.update(&[chap_id]);
                    hasher.update(stored_pass.as_bytes());
                    hasher.update(challenge);
                    let expected_response = hasher.finalize();

                    // Compare with the received response
                    if chap_response == expected_response.as_slice() {
                        Ok(AuthResult::Success)
                    } else {
                        Ok(AuthResult::InvalidPassword)
                    }

                } else {
                    Ok(AuthResult::InvalidPassword)
                }
            }
            None => Ok(AuthResult::UserNotFound),
        }
    }

    async fn authenticate_mschap(&self, username: &str, challenge: &[u8], response: &[u8], authenticator: &[u8], secret: &str) -> Result<AuthResult, sqlx::Error> {
        debug!("Authenticating MS-CHAP:");
        debug!("Challenge length: {}", challenge.len());
        debug!("Response length: {}", response.len());

        let pool = self.auth_server.get_pool();

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
                if !record.is_enabled {
                    return Ok(AuthResult::AccountDisabled);
                }

                if let Some(stored_pass) = record.plain_password {
                    // Convert password to UTF-16LE bytes
                    let utf16_bytes: Vec<u8> = stored_pass.encode_utf16()
                        .flat_map(|x| x.to_le_bytes().to_vec())
                        .collect();

                    // Generate NT hash from password
                    let nt_hash = nt_hash(&utf16_bytes);

                    // Pad the hash to 21 bytes for DES
                    let mut padded_hash = nt_hash.clone();
                    padded_hash.resize(21, 0);

                    // Generate three 8-byte DES keys and encrypt the challenge
                    let mut challenge_response = Vec::with_capacity(24);

                    for i in 0..3 {
                        let key_7 = &padded_hash[i * 7..(i + 1) * 7];
                        let key_8 = setup_des_key(key_7);

                        let cipher = des::Des::new_from_slice(&key_8)
                            .map_err(|_| sqlx::Error::Protocol("Failed to create DES cipher".into()))?;

                        let mut block_array = GenericArray::clone_from_slice(&challenge[..8]);
                        cipher.encrypt_block(&mut block_array);
                        challenge_response.extend_from_slice(&block_array);
                    }

                    // Compare responses
                    if response == &challenge_response {
                        Ok(AuthResult::Success)
                    } else {

                        Ok(AuthResult::InvalidPassword)
                    }
                } else {
                    Ok(AuthResult::InvalidPassword)
                }
            }
            None => Ok(AuthResult::UserNotFound),
        }
    }


    async fn authenticate_mschap2(&self, username: &str, peer_challenge: &[u8], nt_response: &[u8], authenticator: &[u8], secret: &str) -> Result<Mschapv2Result, sqlx::Error> {
        let pool = self.auth_server.get_pool();
        debug!("MS-CHAPv2: Starting authentication for user: {}", username);
        debug!("MS-CHAPv2: Input lengths - peer_challenge: {} bytes, nt_response: {} bytes, authenticator: {} bytes",
               peer_challenge.len(), nt_response.len(), authenticator.len());

        // Validate input lengths
        if peer_challenge.len() != 16 {
            error!("MS-CHAPv2: Invalid peer_challenge length: {} (expected 16)", peer_challenge.len());
            return Err(sqlx::Error::Protocol(format!("Invalid peer_challenge length: {} (expected 16)", peer_challenge.len()).into()));
        }
        if nt_response.len() < 24 {
            error!("MS-CHAPv2: Invalid nt_response length: {} (expected at least 24)", nt_response.len());
            return Err(sqlx::Error::Protocol(format!("Invalid nt_response length: {} (expected at least 24)", nt_response.len()).into()));
        }
        if authenticator.len() != 16 {
            error!("MS-CHAPv2: Invalid authenticator length: {} (expected 16)", authenticator.len());
            return Err(sqlx::Error::Protocol(format!("Invalid authenticator length: {} (expected 16)", authenticator.len()).into()));
        }

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

        debug!("MS-CHAPv2: Database query result - found: {}, enabled: {:?}",
               result.is_some(),
               result.as_ref().map(|r| r.is_enabled));

        match result {
            Some(record) => {
                if !record.is_enabled {
                    debug!("MS-CHAPv2: Account disabled for user: {}", username);
                    return Ok(Mschapv2Result {
                        result: AuthResult::AccountDisabled,
                        authenticator_response: None,
                        password_hash: None,
                    });
                }

                if let Some(stored_pass) = record.plain_password {
                    debug!("MS-CHAPv2: Password found for user: {} (length: {} bytes)", username, stored_pass.len());

                    // Generate NT hash from password
                    let password_utf16: Vec<u8> = stored_pass.encode_utf16()
                        .flat_map(|c| c.to_le_bytes().to_vec())
                        .collect();

                    debug!("MS-CHAPv2: Password UTF-16LE length: {} bytes", password_utf16.len());

                    // Generate NT hash from the UTF-16LE password
                    let password_hash = nt_hash(&password_utf16);
                    debug!("MS-CHAPv2: Generated password hash: {} bytes", password_hash.len());

                    // Generate the challenge using SHA1(peer_challenge + authenticator + username)
                    debug!("MS-CHAPv2: Challenge inputs - peer_challenge: {:02x?}, authenticator: {:02x?}, username: {}", 
                           peer_challenge, authenticator, username);
                    let challenge = generate_nt_response_challenge(peer_challenge, authenticator, username);
                    debug!("MS-CHAPv2: Generated challenge: {} bytes, value: {:02x?}", challenge.len(), challenge);

                    // Generate expected response
                    let expected_response = generate_nt_response(&password_hash, &challenge);
                    debug!("MS-CHAPv2: Generated expected NT-Response: {} bytes", expected_response.len());

                    // Compare only the first 24 bytes (NT-Response is 24 bytes)
                    let received_nt_response = &nt_response[0..24];
                    if received_nt_response == &expected_response[0..24] {
                        debug!("MS-CHAPv2: NT-Response validation successful for user: {}", username);
                        let authenticator_response = Some(calculate_authenticator_response(
                            &password_hash,
                            received_nt_response,
                            peer_challenge,
                            authenticator,
                            username,
                        ));
                        debug!("MS-CHAPv2: Generated authenticator response: {} bytes", authenticator_response.as_ref().map(|r| r.len()).unwrap_or(0));
                        Ok(Mschapv2Result {
                            result: AuthResult::Success,
                            authenticator_response,
                            password_hash: Some(password_hash)
                        })
                    } else {
                        debug!("MS-CHAPv2: NT-Response validation failed for user: {}", username);
                        debug!("MS-CHAPv2: Expected: {:02x?}", &expected_response[0..24]);
                        debug!("MS-CHAPv2: Received: {:02x?}", received_nt_response);
                        Ok(Mschapv2Result {
                            result: AuthResult::InvalidPassword,
                            authenticator_response: None,
                            password_hash: None
                        })
                    }
                } else {
                    debug!("MS-CHAPv2: No password stored for user: {}", username);
                    Ok(Mschapv2Result {
                        result: AuthResult::InvalidPassword,
                        authenticator_response: None,
                        password_hash: None,
                    })
                }
            }
            None => {
                debug!("MS-CHAPv2: User not found in database: {}", username);
                Ok(Mschapv2Result {
                    result: AuthResult::UserNotFound,
                    authenticator_response: None,
                    password_hash: None,
                })
            }
        }
    }

    fn create_access_accept(&self, request: &RadiusPacket, secret: &str) -> Vec<u8> {
        // Create the basic Access-Accept packet
        debug!("Creating Access-Accept response for request: {:?}", request);
        let mut response = RadiusPacket {
            code: 2, // Access-Accept
            identifier: request.identifier,
            length: 20, // The initial length with just header will be updated
            authenticator: request.authenticator,
            attributes: Vec::new(),
        };

        // Add Message-Authenticator if it was in the request
        let has_msg_auth = request.attributes.iter().any(|attr| attr.typ == ATTR_MESSAGE_AUTHENTICATOR);
        if has_msg_auth {
            response.attributes.push(RadiusAttribute {
                typ: ATTR_MESSAGE_AUTHENTICATOR,
                value: vec![0u8; 16], // Will be calculated later
            });
        }

        // Encode the packet to get the complete structure
        let mut encoded = response.encode();

        // 1. Calculate Response Authenticator first
        let mut hasher = md5::Md5::new();
        hasher.update(&encoded[0..4]); // Code+ID+Length
        hasher.update(&request.authenticator); // RequestAuth
        hasher.update(&encoded[20..]); // Attributes
        hasher.update(secret.as_bytes()); // Secret
        let response_auth = hasher.finalize();

        // Update Response Authenticator in the packet
        encoded[4..20].copy_from_slice(&response_auth);

        // 2. Calculate Message-Authenticator if present
        if has_msg_auth {
            if let Some(pos) = encoded.windows(2).position(|w| w[0] == ATTR_MESSAGE_AUTHENTICATOR) {
                // Create a temporary packet with zeroed Message-Authenticator
                let mut temp_packet = encoded.clone();
                for i in 0..16 {
                    temp_packet[pos + 2 + i] = 0;
                }

                // Calculate HMAC-MD5 over the entire packet
                let mut mac = <Hmac<Md5> as Mac>::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(&temp_packet);
                let result = mac.finalize();
                let message_auth = result.into_bytes();

                // Update Message-Authenticator in the packet
                encoded[pos + 2..pos + 18].copy_from_slice(&message_auth);
                debug!("Message-Authenticator calculated: {:?}", message_auth);

                // Verify the Message-Authenticator is not zeroed out
                let is_zeroed = encoded[pos + 2..pos + 18].iter().all(|&b| b == 0);
                if is_zeroed {
                    error!("Message-Authenticator is zeroed out after calculation in create_access_accept!");
                }
            }
        }

        // Debug the final encoded packet
        debug!("Final encoded packet: {:?}", encoded);

        encoded
    }

    fn create_access_accept_mschapv2(
        &self,
        request: &RadiusPacket,
        secret: &str,
        ms_chap_v2_ident: u8,
        authenticator_response: &[u8],
        password_hash: &[u8],
        nt_response: &[u8],
    ) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        use md5::Md5;
        type HmacMd5 = Hmac<Md5>;

        let mut response = RadiusPacket {
            code: 2, // Access-Accept
            identifier: request.identifier,
            length: 20,
            authenticator: request.authenticator,
            attributes: Vec::new(),
        };

        debug!("Creating Access-Accept for MS-CHAPv2");
        debug!("Secret: {}", secret);

        // MS-CHAP2-Success (format: 1 byte identifier | ASCII("S=" + 40-char hex))
        // Authenticator response is 20 bytes, which becomes 40 hex characters
        assert_eq!(authenticator_response.len(), 20, "Authenticator response must be exactly 20 bytes");
        let hex = authenticator_response.iter().map(|b| format!("{:02X}", b)).collect::<String>();
        let success_str = format!("S={}", hex);
        let mut ms_chap_success = vec![ms_chap_v2_ident];
        ms_chap_success.extend_from_slice(success_str.as_bytes());

        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: [
                &VENDOR_MICROSOFT.to_be_bytes()[..],
                &[VENDOR_ATTR_MS_CHAP2_SUCCESS, (ms_chap_success.len() + 2) as u8],
                &ms_chap_success[..]
            ].concat(),
        });

        // MS-MPPE-Encryption-Policy
        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: [
                &VENDOR_MICROSOFT.to_be_bytes()[..],
                &[VENDOR_ATTR_MS_MPPE_ENCRYPTION_POLICY, 6],
                &[0, 0, 0, 1],
            ]
                .concat(),
        });

        // MS-MPPE-Encryption-Types
        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: [
                &VENDOR_MICROSOFT.to_be_bytes()[..],
                &[VENDOR_ATTR_MS_MPPE_ENCRYPTION_TYPES, 6],
                &[0, 0, 0, 6],
            ]
                .concat(),
        });

        // Session keys
        let (send_key, recv_key) = Self::get_mschapv2_session_keys(password_hash, nt_response);
        let enc_send = Self::encrypt_mppe_key(&send_key, secret, &request.authenticator);
        let enc_recv = Self::encrypt_mppe_key(&recv_key, secret, &request.authenticator);

        // Send Key
        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: [
                &VENDOR_MICROSOFT.to_be_bytes()[..],
                &[VENDOR_ATTR_MS_MPPE_SEND_KEY, (enc_send.len() + 2) as u8],
                &enc_send[..],
            ]
                .concat(),
        });

        // Recv Key
        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: [
                &VENDOR_MICROSOFT.to_be_bytes()[..],
                &[VENDOR_ATTR_MS_MPPE_RECV_KEY, (enc_recv.len() + 2) as u8],
                &enc_recv[..],
            ]
                .concat(),
        });

        // Message-Authenticator placeholder
        response.attributes.push(RadiusAttribute {
            typ: ATTR_MESSAGE_AUTHENTICATOR,
            value: vec![0u8; 16],
        });

        // === Encode and patch ===
        let mut encoded = response.encode();

        encoded[4..20].copy_from_slice(&request.authenticator);

        // Find Message-Authenticator
        let mut msg_auth_pos = None;
        let mut pos = 20;
        while pos + 2 <= encoded.len() {
            let typ = encoded[pos];
            let len = encoded[pos + 1] as usize;
            if typ == ATTR_MESSAGE_AUTHENTICATOR && len == 18 {
                msg_auth_pos = Some(pos);
                break;
            }
            if len < 2 || pos + len > encoded.len() {
                break;
            }
            pos += len;
        }

        if let Some(pos) = msg_auth_pos {
            let mut temp_for_mac = encoded.clone();
            for i in 0..16 {
                temp_for_mac[pos + 2 + i] = 0;
            }
            temp_for_mac[4..20].copy_from_slice(&request.authenticator);

            // Calculate Message-Authenticator = HMAC-MD5 over the whole packet with the secret
            let mut mac = <HmacMd5 as hmac::digest::KeyInit>::new_from_slice(secret.as_bytes()).unwrap();

            mac.update(&temp_for_mac);
            let msg_auth = mac.finalize().into_bytes();
            // Patch into our encoded buffer (which still has a request authenticator)
            encoded[pos + 2..pos + 18].copy_from_slice(&msg_auth);

            // --- PASS 3: Now compute the final Response Authenticator

            // Build buffer for Response Authenticator calculation:
            let mut temp_for_auth = encoded.clone();
            // Set authenticator to request authenticator (per RFC) for calculation
            temp_for_auth[4..20].copy_from_slice(&request.authenticator);

            // Compute correct Response Authenticator
            let mut md5 = Md5::new();
            md5.update(&temp_for_auth[0..4]);
            md5.update(&request.authenticator);
            md5.update(&temp_for_auth[20..]);
            md5.update(secret.as_bytes());
            let response_auth = md5.finalize();

            // Patch Response Authenticator into header of the final output
            encoded[4..20].copy_from_slice(&response_auth);

            debug!("✅ Final Response Authenticator: {:02X?}", response_auth);
            debug!("✅ Final Message-Authenticator: {:02X?}", msg_auth);

            encoded

        } else {
            error!("❌ Message-Authenticator not found in packet!");
            encoded
        }


    }

    fn add_mppe_attributes(
        &self,
        response: &mut RadiusPacket,
        secret: &str,
        authenticator: &[u8],
        password_hash: &[u8],
        nt_response: &[u8],
    ) {
        // Get session keys
        let (send_key, recv_key) = Self::get_mschapv2_session_keys(password_hash, nt_response);

        debug!("MPPE Send Key length: {}", send_key.len());
        debug!("MPPE Recv Key length: {}", recv_key.len());

        // Encrypt the keys
        let encrypted_send_key = Self::encrypt_mppe_key(&send_key, secret, authenticator);
        let encrypted_recv_key = Self::encrypt_mppe_key(&recv_key, secret, authenticator);

        debug!("Encrypted Send Key length: {}", encrypted_send_key.len());
        debug!("Encrypted Recv Key length: {}", encrypted_recv_key.len());

        // Add MPPE-Send-Key
        let mut send_key_attr = Vec::new();
        send_key_attr.extend_from_slice(&VENDOR_MICROSOFT.to_be_bytes());
        send_key_attr.push(VENDOR_ATTR_MS_MPPE_SEND_KEY); // Vendor-Type: MPPE-Send-Key

        // The encrypted key is already truncated if necessary in encrypt_mppe_key
        debug!("Adding MS-MPPE-Send-Key, encrypted size: {} bytes", encrypted_send_key.len());
        send_key_attr.push((encrypted_send_key.len() + 2) as u8); // Vendor-Length
        send_key_attr.extend_from_slice(&encrypted_send_key);

        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: send_key_attr,
        });

        // Add MPPE-Recv-Key
        let mut recv_key_attr = Vec::new();
        recv_key_attr.extend_from_slice(&VENDOR_MICROSOFT.to_be_bytes());
        recv_key_attr.push(VENDOR_ATTR_MS_MPPE_RECV_KEY); // Vendor-Type: MPPE-Recv-Key

        // The encrypted key is already truncated if necessary in encrypt_mppe_key
        debug!("Adding MS-MPPE-Recv-Key, encrypted size: {} bytes", encrypted_recv_key.len());
        recv_key_attr.push((encrypted_recv_key.len() + 2) as u8); // Vendor-Length
        recv_key_attr.extend_from_slice(&encrypted_recv_key);

        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: recv_key_attr,
        });

        // Add MPPE-Encryption-Policy
        let mut policy_attr = Vec::new();
        policy_attr.extend_from_slice(&VENDOR_MICROSOFT.to_be_bytes());
        policy_attr.push(VENDOR_ATTR_MS_MPPE_ENCRYPTION_POLICY); // Vendor-Type: MPPE-Encryption-Policy
        policy_attr.push(6); // Vendor-Length
        policy_attr.extend_from_slice(&[0, 0, 0, 1]); // Value: 1 (Required)

        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: policy_attr,
        });

        // Add MPPE-Encryption-Types
        let mut types_attr = Vec::new();
        types_attr.extend_from_slice(&VENDOR_MICROSOFT.to_be_bytes());
        types_attr.push(VENDOR_ATTR_MS_MPPE_ENCRYPTION_TYPES); // Vendor-Type: MPPE-Encryption-Types
        types_attr.push(6); // Vendor-Length
        types_attr.extend_from_slice(&[0, 0, 0, 6]); // Value: 6 (RC4)

        response.attributes.push(RadiusAttribute {
            typ: ATTR_VENDOR_SPECIFIC,
            value: types_attr,
        });
    }

    async fn handle_eap_request(&self, packet: &RadiusPacket, secret: &str) -> Vec<u8> {
        // Extract EAP message from attributes
        let eap_data = packet.attributes.iter()
            .find(|attr| attr.typ == ATTR_EAP_MESSAGE)
            .map(|attr| attr.value.clone());

        if let Some(eap_data) = eap_data {
            if let Some(eap_packet) = EapPacket::parse(&eap_data) {
                match eap_packet.type_ {
                    EAP_TYPE_TLS => {
                        match self.handle_eap_tls(packet, &eap_packet, secret).await {
                            Ok(response) => response,
                            Err(e) => self.create_access_reject(packet, secret, &format!("EAP-TLS error: {}", e))
                        }
                    }
                    EAP_TYPE_TTLS => {
                        match self.handle_eap_ttls(packet, &eap_packet, secret).await {
                            Ok(response) => response,
                            Err(e) => self.create_access_reject(packet, secret, &format!("EAP-TTLS error: {}", e))
                        }
                    }
                    EAP_TYPE_PEAP => {
                        match self.handle_eap_peap(packet, &eap_packet, secret).await {
                            Ok(response) => response,
                            Err(e) => self.create_access_reject(packet, secret, &format!("EAP-PEAP error: {}", e))
                        }
                    }
                    EAP_TYPE_SIM => {
                        self.handle_eap_sim(packet, &eap_packet, secret).await
                    }
                    EAP_TYPE_AKA => {
                        self.handle_eap_aka(packet, &eap_packet, secret).await
                    }
                    EAP_TYPE_AKA_PRIME => {
                        self.handle_eap_aka_prime(packet, &eap_packet, secret).await
                    }
                    _ => {
                        self.create_access_reject(packet, secret, "Unsupported EAP method")
                    }
                }
            } else {
                self.create_access_reject(packet, secret, "Invalid EAP packet")
            }
        } else {
            self.create_access_reject(packet, secret, "Missing EAP message")
        }
    }

    async fn handle_eap_sim(&self, packet: &RadiusPacket, eap_packet: &EapPacket, secret: &str) -> Vec<u8> {
        // Extract IMSI from EAP identity
        let imsi = if eap_packet.code == EAP_RESPONSE && eap_packet.type_ == 1 {
            // EAP Identity response
            String::from_utf8_lossy(&eap_packet.data).to_string()
        } else {
            // Try to find IMSI in RADIUS attributes
            packet.attributes.iter()
                .find(|attr| attr.typ == ATTR_USER_NAME)
                .map(|attr| String::from_utf8_lossy(&attr.value).to_string())
                .unwrap_or_default()
        };

        if imsi.is_empty() {
            return self.create_access_reject(packet, secret, "Missing IMSI");
        }

        // Parse EAP-SIM attributes
        let sim_attrs = EapSimAttributes {
            version_list: None,
            selected_version: None,
            nonce_mt: None,
            nonce_s: None,
            rand: None,
            mac: None,
            encr_data: None,
            iv: None,
            next_pseudonym: None,
            next_reauth_id: None,
            result_ind: None,
            counter: None,
            counter_too_small: None,
            notification: None,
            client_error_code: None,
        };

        // TODO: Parse EAP-SIM attributes from eap_packet.data

        match eap_packet.code {
            EAP_REQUEST => {
                // Start EAP-SIM authentication
                let response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_SIM,
                    data: vec![EAP_SIM_START], // Start subtype
                };

                // Create RADIUS response with EAP message
                let radius_response = RadiusPacket {
                    code: 11, // Access-Challenge
                    identifier: packet.identifier,
                    length: 20,
                    authenticator: packet.authenticator,
                    attributes: vec![
                        RadiusAttribute {
                            typ: ATTR_EAP_MESSAGE,
                            value: response.encode(),
                        },
                        RadiusAttribute {
                            typ: ATTR_MESSAGE_AUTHENTICATOR,
                            value: vec![0u8; 16],
                        },
                    ],
                };

                // Calculate Message-Authenticator
                let mut encoded = radius_response.encode();
                let msg_auth_pos = encoded.windows(2)
                    .position(|w| w[0] == ATTR_MESSAGE_AUTHENTICATOR)
                    .unwrap();

                let mut mac = <Hmac<Md5> as Mac>::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(&encoded[0..msg_auth_pos + 2]);
                mac.update(&[0u8; 16]);
                mac.update(&encoded[msg_auth_pos + 18..]);
                let result = mac.finalize();
                encoded[msg_auth_pos + 2..msg_auth_pos + 18].copy_from_slice(&result.into_bytes());

                encoded
            }
            EAP_RESPONSE => {
                // Continue EAP-SIM authentication
                if eap_packet.data.is_empty() {
                    return self.create_access_reject(packet, secret, "Empty EAP-SIM data");
                }

                // TODO: Implement EAP-SIM authentication state machine
                // This requires:
                // 1. SIM card authentication
                // 2. Triple authentication vectors
                // 3. Session key derivation
                self.create_access_reject(packet, secret, "EAP-SIM authentication not implemented")
            }
            _ => {
                self.create_access_reject(packet, secret, "Invalid EAP code")
            }
        }
    }

    async fn handle_eap_aka(&self, packet: &RadiusPacket, eap_packet: &EapPacket, secret: &str) -> Vec<u8> {
        // Extract IMSI from EAP identity
        let imsi = if eap_packet.code == EAP_RESPONSE && eap_packet.type_ == 1 {
            // EAP Identity response
            String::from_utf8_lossy(&eap_packet.data).to_string()
        } else {
            // Try to find IMSI in RADIUS attributes
            packet.attributes.iter()
                .find(|attr| attr.typ == ATTR_USER_NAME)
                .map(|attr| String::from_utf8_lossy(&attr.value).to_string())
                .unwrap_or_default()
        };

        if imsi.is_empty() {
            return self.create_access_reject(packet, secret, "Missing IMSI");
        }

        // Parse EAP-AKA attributes
        let aka_attrs = EapAkaAttributes {
            rand: None,
            autn: None,
            ik: None,
            ck: None,
            res: None,
            auts: None,
            next_pseudonym: None,
            next_reauth_id: None,
            result_ind: None,
            counter: None,
            counter_too_small: None,
            notification: None,
            client_error_code: None,
        };

        // TODO: Parse EAP-AKA attributes from eap_packet.data

        match eap_packet.code {
            EAP_REQUEST => {
                // Start EAP-AKA authentication
                let response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_AKA,
                    data: vec![EAP_AKA_IDENTITY], // Identity subtype
                };

                // Create RADIUS response with EAP message
                let radius_response = RadiusPacket {
                    code: 11, // Access-Challenge
                    identifier: packet.identifier,
                    length: 20,
                    authenticator: packet.authenticator,
                    attributes: vec![
                        RadiusAttribute {
                            typ: ATTR_EAP_MESSAGE,
                            value: response.encode(),
                        },
                        RadiusAttribute {
                            typ: ATTR_MESSAGE_AUTHENTICATOR,
                            value: vec![0u8; 16],
                        },
                    ],
                };

                // Calculate Message-Authenticator
                let mut encoded = radius_response.encode();
                let msg_auth_pos = encoded.windows(2)
                    .position(|w| w[0] == ATTR_MESSAGE_AUTHENTICATOR)
                    .unwrap();

                let mut mac = <Hmac<Md5> as Mac>::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(&encoded[0..msg_auth_pos + 2]);
                mac.update(&[0u8; 16]);
                mac.update(&encoded[msg_auth_pos + 18..]);
                let result = mac.finalize();
                encoded[msg_auth_pos + 2..msg_auth_pos + 18].copy_from_slice(&result.into_bytes());

                encoded
            }
            EAP_RESPONSE => {
                // Continue EAP-AKA authentication
                if eap_packet.data.is_empty() {
                    return self.create_access_reject(packet, secret, "Empty EAP-AKA data");
                }

                // TODO: Implement EAP-AKA authentication state machine
                // This requires:
                // 1. USIM card authentication
                // 2. Authentication vectors (RAND, AUTN, XRES, CK, IK)
                // 3. Session key derivation
                self.create_access_reject(packet, secret, "EAP-AKA authentication not implemented")
            }
            _ => {
                self.create_access_reject(packet, secret, "Invalid EAP code")
            }
        }
    }

    async fn handle_eap_aka_prime(&self, packet: &RadiusPacket, eap_packet: &EapPacket, secret: &str) -> Vec<u8> {
        // EAP-AKA' is similar to EAP-AKA but with additional key derivation
        // We can reuse most of the EAP-AKA code with some modifications
        self.handle_eap_aka(packet, eap_packet, secret).await
    }

    async fn handle_eap_tls(&self, packet: &RadiusPacket, eap_packet: &EapPacket, secret: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Extract username from EAP identity
        let username = if eap_packet.code == EAP_RESPONSE && eap_packet.type_ == 1 {
            // EAP Identity response
            String::from_utf8_lossy(&eap_packet.data).to_string()
        } else {
            // Try to find username in RADIUS attributes
            packet.attributes.iter()
                .find(|attr| attr.typ == ATTR_USER_NAME)
                .map(|attr| String::from_utf8_lossy(&attr.value).to_string())
                .unwrap_or_default()
        };

        if username.is_empty() {
            return Ok(self.create_access_reject(packet, secret, "Missing username"));
        }

        // Load server certificate and private key
        let cert_file = File::open("certs/server.crt")
            .map_err(|_| "Failed to open certificate file")?;

        let key_file = File::open("certs/server.key")
            .map_err(|_| "Failed to open private key file")?;

        let mut reader = BufReader::new(cert_file);

        let cert_chain = match certs(&mut reader) {
            Ok(certs) => certs.into_iter().map(Certificate).collect::<Vec<_>>(),
            Err(_) => return Ok(self.create_access_reject(packet, secret, "Failed to parse certificate")),
        };
        let mut key_reader = BufReader::new(key_file);
        let mut keys = match pkcs8_private_keys(&mut key_reader) {
            Ok(keys) => keys,
            Err(_) => return Ok(self.create_access_reject(packet, secret, "Failed to parse private key")),
        };

        if keys.is_empty() {
            return Ok(self.create_access_reject(packet, secret, "No private key found"));
        }

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                cert_chain,
                PrivateKey(keys.remove(0))
            )
            .map_err(|_| "Failed to create TLS config")?;

        let acceptor = TlsAcceptor::from(Arc::new(config));

        match eap_packet.code {
            EAP_REQUEST => {
                // Start EAP-TLS handshake
                let response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_TLS,
                    data: vec![0x80], // Start bit set
                };

                // Create RADIUS response with EAP message
                let radius_response = RadiusPacket {
                    code: 11, // Access-Challenge
                    identifier: packet.identifier,
                    length: 20,
                    authenticator: packet.authenticator,
                    attributes: vec![
                        RadiusAttribute {
                            typ: ATTR_EAP_MESSAGE,
                            value: response.encode(),
                        },
                        RadiusAttribute {
                            typ: ATTR_MESSAGE_AUTHENTICATOR,
                            value: vec![0u8; 16],
                        },
                    ],
                };

                // Calculate Message-Authenticator
                let mut encoded = radius_response.encode();
                let msg_auth_pos = encoded.windows(2)
                    .position(|w| w[0] == ATTR_MESSAGE_AUTHENTICATOR)
                    .unwrap();

                let mut mac = <Hmac<Md5> as Mac>::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(&encoded[0..msg_auth_pos + 2]);
                mac.update(&[0u8; 16]);
                mac.update(&encoded[msg_auth_pos + 18..]);
                let result = mac.finalize();
                encoded[msg_auth_pos + 2..msg_auth_pos + 18].copy_from_slice(&result.into_bytes());

                Ok(encoded)
            }
            EAP_RESPONSE => {
                // Continue EAP-TLS handshake
                if eap_packet.data.is_empty() {
                    return Ok(self.create_access_reject(packet, secret, "Empty EAP-TLS data"));
                }

                // TODO: Implement EAP-TLS handshake state machine
                // This requires:
                // 1. TLS handshake state tracking
                // 2. Certificate validation
                // 3. Session key derivation
                Ok(self.create_access_reject(packet, secret, "EAP-TLS handshake not implemented"))
            }
            _ => {
                Ok(self.create_access_reject(packet, secret, "Invalid EAP code"))
            }
        }
    }

    async fn handle_eap_peap(&self, packet: &RadiusPacket, eap_packet: &EapPacket, secret: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Extract username from EAP identity
        let username = if eap_packet.code == EAP_RESPONSE && eap_packet.type_ == 1 {
            // EAP Identity response
            String::from_utf8_lossy(&eap_packet.data).to_string()
        } else {
            // Try to find username in RADIUS attributes
            packet.attributes.iter()
                .find(|attr| attr.typ == ATTR_USER_NAME)
                .map(|attr| String::from_utf8_lossy(&attr.value).to_string())
                .unwrap_or_default()
        };

        if username.is_empty() {
            return Ok(self.create_access_reject(packet, secret, "Missing username"));
        }

        // Load server certificate and private key
        let cert_file = File::open("certs/server.crt")
            .map_err(|_| "Failed to open certificate file")?;

        let key_file = File::open("certs/server.key")
            .map_err(|_| "Failed to open private key file")?;

        let mut reader = BufReader::new(cert_file);

        let cert_chain = match certs(&mut reader) {
            Ok(certs) => certs.into_iter().map(Certificate).collect::<Vec<_>>(),
            Err(_) => return Ok(self.create_access_reject(packet, secret, "Failed to parse certificate")),
        };
        let mut key_reader = BufReader::new(key_file);
        let mut keys = match pkcs8_private_keys(&mut key_reader) {
            Ok(keys) => keys,
            Err(_) => return Ok(self.create_access_reject(packet, secret, "Failed to parse private key")),
        };

        if keys.is_empty() {
            return Ok(self.create_access_reject(packet, secret, "No private key found"));
        }

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                cert_chain,
                PrivateKey(keys.remove(0))
            )
            .map_err(|_| "Failed to create TLS config")?;

        let acceptor = TlsAcceptor::from(Arc::new(config));

        match eap_packet.code {
            EAP_REQUEST => {
                // Start PEAP handshake
                let response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_PEAP,
                    data: vec![0x80], // Start bit set
                };

                // Create RADIUS response with EAP message
                let radius_response = RadiusPacket {
                    code: 11, // Access-Challenge
                    identifier: packet.identifier,
                    length: 20,
                    authenticator: packet.authenticator,
                    attributes: vec![
                        RadiusAttribute {
                            typ: ATTR_EAP_MESSAGE,
                            value: response.encode(),
                        },
                        RadiusAttribute {
                            typ: ATTR_MESSAGE_AUTHENTICATOR,
                            value: vec![0u8; 16],
                        },
                    ],
                };

                // Calculate Message-Authenticator
                let mut encoded = radius_response.encode();
                let msg_auth_pos = encoded.windows(2)
                    .position(|w| w[0] == ATTR_MESSAGE_AUTHENTICATOR)
                    .unwrap();

                let mut mac = <Hmac<Md5> as Mac>::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(&encoded[0..msg_auth_pos + 2]);
                mac.update(&[0u8; 16]);
                mac.update(&encoded[msg_auth_pos + 18..]);
                let result = mac.finalize();
                encoded[msg_auth_pos + 2..msg_auth_pos + 18].copy_from_slice(&result.into_bytes());

                Ok(encoded)
            }
            EAP_RESPONSE => {
                // Continue PEAP handshake
                if eap_packet.data.is_empty() {
                    return Ok(self.create_access_reject(packet, secret, "Empty PEAP data"));
                }

                // TODO: Implement PEAP handshake state machine
                // This requires:
                // 1. TLS handshake state tracking
                // 2. Certificate validation
                // 3. MS-CHAPv2 inner authentication
                // 4. Session key derivation
                Ok(self.create_access_reject(packet, secret, "PEAP handshake not implemented"))
            }
            _ => {
                Ok(self.create_access_reject(packet, secret, "Invalid EAP code"))
            }
        }
    }

    async fn handle_eap_ttls(&self, packet: &RadiusPacket, eap_packet: &EapPacket, secret: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Extract username from EAP identity
        let username = if eap_packet.code == EAP_RESPONSE && eap_packet.type_ == 1 {
            // EAP Identity response
            String::from_utf8_lossy(&eap_packet.data).to_string()
        } else {
            // Try to find username in RADIUS attributes
            packet.attributes.iter()
                .find(|attr| attr.typ == ATTR_USER_NAME)
                .map(|attr| String::from_utf8_lossy(&attr.value).to_string())
                .unwrap_or_default()
        };

        if username.is_empty() {
            return Ok(self.create_access_reject(packet, secret, "Missing username"));
        }

        // Load server certificate and private key
        let cert_file = File::open("certs/server.crt")
            .map_err(|_| "Failed to open certificate file")?;

        let key_file = File::open("certs/server.key")
            .map_err(|_| "Failed to open private key file")?;

        let mut reader = BufReader::new(cert_file);

        let cert_chain = match certs(&mut reader) {
            Ok(certs) => certs.into_iter().map(Certificate).collect::<Vec<_>>(),
            Err(_) => return Ok(self.create_access_reject(packet, secret, "Failed to parse certificate")),
        };
        let mut key_reader = BufReader::new(key_file);
        let mut keys = match pkcs8_private_keys(&mut key_reader) {
            Ok(keys) => keys,
            Err(_) => return Ok(self.create_access_reject(packet, secret, "Failed to parse private key")),
        };

        if keys.is_empty() {
            return Ok(self.create_access_reject(packet, secret, "No private key found"));
        }

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                cert_chain,
                PrivateKey(keys.remove(0))
            )
            .map_err(|_| "Failed to create TLS config")?;

        match eap_packet.code {
            EAP_REQUEST => {
                // Start TTLS handshake
                let response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_TTLS,
                    data: vec![0x80], // Start bit set
                };

                // Create RADIUS response with EAP message
                let mut radius_response = RadiusPacket {
                    code: 11, // Access-Challenge
                    identifier: packet.identifier,
                    length: 20,
                    authenticator: packet.authenticator,
                    attributes: vec![
                        RadiusAttribute {
                            typ: ATTR_EAP_MESSAGE,
                            value: response.encode(),
                        },
                        RadiusAttribute {
                            typ: ATTR_MESSAGE_AUTHENTICATOR,
                            value: vec![0u8; 16],
                        },
                    ],
                };

                // Calculate Message-Authenticator
                let mut encoded = radius_response.encode();
                let msg_auth_pos = encoded.windows(2)
                    .position(|w| w[0] == ATTR_MESSAGE_AUTHENTICATOR)
                    .unwrap();

                let mut mac = <Hmac<Md5> as Mac>::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(&encoded[0..msg_auth_pos + 2]);
                mac.update(&[0u8; 16]);
                mac.update(&encoded[msg_auth_pos + 18..]);
                let result = mac.finalize();
                encoded[msg_auth_pos + 2..msg_auth_pos + 18].copy_from_slice(&result.into_bytes());

                Ok(encoded)
            }
            EAP_RESPONSE => {
                // Continue TTLS handshake
                if eap_packet.data.is_empty() {
                    return Ok(self.create_access_reject(packet, secret, "Empty TTLS data"));
                }

                // TODO: Implement TTLS handshake state machine
                // This requires:
                // 1. TLS handshake state tracking
                // 2. Certificate validation
                // 3. Inner authentication method selection and handling
                // 4. Session key derivation
                Ok(self.create_access_reject(packet, secret, "TTLS handshake not implemented"))
            }
            _ => {
                Ok(self.create_access_reject(packet, secret, "Invalid EAP code"))
            }
        }
    }

    fn create_accounting_response(&self, request: &RadiusPacket) -> Vec<u8> {
        let response = RadiusPacket {
            code: 5, // Accounting-Response
            identifier: request.identifier,
            length: 0, // Will be set after adding attributes
            authenticator: [0u8; 16], // Will be set after adding attributes
            attributes: Vec::new(),
        };

        // Encode the response
        let encoded = response.encode();

        encoded
    }

    fn get_mschapv2_session_keys(password_hash: &[u8], nt_response: &[u8]) -> (Vec<u8>, Vec<u8>) {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;

        let magic1 = b"This is the MPPE Master Key";
        let magic2 = b"On the client side, this is the send key; on the server side, it is the receive key.";
        let magic3 = b"On the client side, this is the receive key; on the server side, it is the send key.";

        // MasterKey = SHA1(PwhashHash, NTResponse, Magic1)
        let mut sha1 = Sha1::new();
        sha1.update(password_hash);
        sha1.update(nt_response);
        sha1.update(magic1);
        let master_key = sha1.finalize();

        // SendKey = SHA1(MasterKey, Magic2)[0..16]
        let mut sha1 = Sha1::new();
        sha1.update(&master_key);
        sha1.update(magic2);
        let send_key = sha1.finalize()[..16].to_vec();

        // RecvKey = SHA1(MasterKey, Magic3)[0..16]
        let mut sha1 = Sha1::new();
        sha1.update(&master_key);
        sha1.update(magic3);
        let recv_key = sha1.finalize()[..16].to_vec();

        (send_key, recv_key)
    }

    fn encrypt_mppe_key(key: &[u8], secret: &str, authenticator: &[u8]) -> Vec<u8> {
        use md5::Md5;
        use rand::random;

        // Maximum size for the encrypted key (253 bytes for VSA value - 6 bytes for VSA header)
        const MAX_ENCRYPTED_KEY_SIZE: usize = 247;

        // Generate salt
        let salt_value: u32 = random();
        let salt = [
            0x80 | ((0 & 0x0f) << 3) | ((salt_value & 0x07) as u8),
            (salt_value & 0xff) as u8
        ];

        // Calculate maximum key size that can be encrypted without exceeding the limit
        // Each 16-byte block of the key becomes 16 bytes of encrypted data
        // We need to reserve 2 bytes for the salt
        let max_key_blocks = (MAX_ENCRYPTED_KEY_SIZE - 2) / 16;
        let max_key_size = max_key_blocks * 16;

        // Truncate the key if necessary
        let key_to_use = if key.len() > max_key_size {
            debug!("Key size {} exceeds maximum allowed size, truncating to {} bytes", key.len(), max_key_size);
            &key[0..max_key_size]
        } else {
            key
        };

        // Pad key to multiple of 16 bytes
        let mut padded_key = key_to_use.to_vec();
        let pad_len = 16 - (padded_key.len() % 16);
        if pad_len < 16 {
            padded_key.extend(vec![0u8; pad_len]);
        }

        // First block
        let mut hasher = Md5::new();
        hasher.update(secret.as_bytes());
        hasher.update(authenticator);
        hasher.update(&salt);
        let first_hash = hasher.finalize();

        let mut result = Vec::with_capacity(2 + padded_key.len());
        result.extend_from_slice(&salt);

        // XOR first block
        let mut encrypted = Vec::new();
        for (a, b) in padded_key[0..16].iter().zip(first_hash.iter()) {
            encrypted.push(a ^ b);
        }
        result.extend_from_slice(&encrypted);

        // Remaining blocks
        for chunk in padded_key[16..].chunks(16) {
            let mut hasher = Md5::new();
            hasher.update(secret.as_bytes());
            hasher.update(&encrypted);
            let hash = hasher.finalize();

            encrypted = Vec::new();
            for (a, b) in chunk.iter().zip(hash.iter()) {
                encrypted.push(a ^ b);
            }
            result.extend_from_slice(&encrypted);
        }

        // Final check to ensure we don't exceed the maximum size
        if result.len() > MAX_ENCRYPTED_KEY_SIZE {
            error!("Encrypted key size {} exceeds maximum allowed size of {} bytes, truncating", result.len(), MAX_ENCRYPTED_KEY_SIZE);
            result.truncate(MAX_ENCRYPTED_KEY_SIZE);
        }

        result
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

// Helper function to generate the challenge for NT-Response
// According to RFC 2759, ChallengeHash = SHA1(PeerChallenge || AuthenticatorChallenge || UserName)[0..8]
fn generate_nt_response_challenge(peer_challenge: &[u8], authenticator_challenge: &[u8], username: &str) -> Vec<u8> {
    use sha1::{Sha1, Digest};

    let mut hasher = Sha1::new();
    // The challenge should use peer challenge first, then authenticator challenge, then username
    // Username should be in ASCII/UTF-8 encoding (not UTF-16LE)
    hasher.update(peer_challenge);
    hasher.update(authenticator_challenge);
    hasher.update(username.as_bytes());
    let hash = hasher.finalize();
    // Take first 8 bytes as the challenge
    hash[0..8].to_vec()
}

fn generate_nt_response(password_hash: &[u8], challenge: &[u8]) -> Vec<u8> {
    let mut padded_hash = password_hash.to_vec();
    padded_hash.resize(21, 0);

    let mut response = Vec::with_capacity(24);

    // Generate three DES keys from the 21-byte padded hash
    for i in 0..3 {
        let start = i * 7;
        let mut key = [0u8; 8];

        // Convert 7 bytes into 8 bytes for DES key
        key[0] = padded_hash[start] >> 1;
        key[1] = ((padded_hash[start] & 0x01) << 6) | (padded_hash[start + 1] >> 2);
        key[2] = ((padded_hash[start + 1] & 0x03) << 5) | (padded_hash[start + 2] >> 3);
        key[3] = ((padded_hash[start + 2] & 0x07) << 4) | (padded_hash[start + 3] >> 4);
        key[4] = ((padded_hash[start + 3] & 0x0F) << 3) | (padded_hash[start + 4] >> 5);
        key[5] = ((padded_hash[start + 4] & 0x1F) << 2) | (padded_hash[start + 5] >> 6);
        key[6] = ((padded_hash[start + 5] & 0x3F) << 1) | (padded_hash[start + 6] >> 7);
        key[7] = padded_hash[start + 6] & 0x7F;

        // Left shift each byte by 1 bit
        for b in &mut key {
            *b = (*b << 1) & 0xFE;
        }

        let cipher = des::Des::new_from_slice(&key)
            .expect("Failed to create DES cipher");

        let mut block = GenericArray::clone_from_slice(&challenge[0..8]);
        cipher.encrypt_block(&mut block);
        response.extend_from_slice(&block);
    }

    response
}


fn nt_hash(password: &[u8]) -> Vec<u8> {
    use md4::{Md4, Digest};

    let mut hasher = Md4::new();
    hasher.update(password);
    hasher.finalize().to_vec()
}


/// Convert 7-byte array into 8-byte DES key (with parity bits)
fn setup_des_key(key_7: &[u8]) -> [u8; 8] {
    let mut key = [0u8; 8];
    key[0] = key_7[0];
    key[1] = (key_7[0] << 7) | (key_7[1] >> 1);
    key[2] = (key_7[1] << 6) | (key_7[2] >> 2);
    key[3] = (key_7[2] << 5) | (key_7[3] >> 3);
    key[4] = (key_7[3] << 4) | (key_7[4] >> 4);
    key[5] = (key_7[4] << 3) | (key_7[5] >> 5);
    key[6] = (key_7[5] << 2) | (key_7[6] >> 6);
    key[7] = key_7[6] << 1;
    key
}

// Helper function to convert a string to UTF-16LE bytes
trait ToUtf16Le {
    fn to_utf16le(&self) -> Vec<u8>;
}

impl ToUtf16Le for str {
    fn to_utf16le(&self) -> Vec<u8> {
        self.encode_utf16()
            .flat_map(|c| c.to_le_bytes().to_vec())
            .collect()
    }
}

fn calculate_authenticator_response(
    password_hash: &[u8],
    nt_response: &[u8],
    peer_challenge: &[u8],
    authenticator_challenge: &[u8],
    username: &str,
) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;

    // Generate PasswordHashHash = SHA1(password_hash)
    let mut sha1 = Sha1::new();
    sha1.update(password_hash);
    let password_hash_hash = sha1.finalize();

    // Generate ChallengeHash = SHA1(PeerChallenge + AuthenticatorChallenge + Username)[0..8]
    let mut sha1 = Sha1::new();
    sha1.update(peer_challenge);
    sha1.update(authenticator_challenge);
    sha1.update(username.as_bytes());
    let challenge_hash = sha1.finalize();

    // Generate the Authenticator Response
    // According to RFC 2759 Section 8.7, the authenticator response is:
    // SHA1(PasswordHashHash, NT-Response, Magic1, ChallengeHash[0..8], Magic2)
    // This produces a single 20-byte value (SHA1 output)
    let magic1 = b"Magic server to client signing constant";
    let magic2 = b"Pad to make it do more than one iteration";

    // Single HMAC-SHA1 calculation combining all inputs
    let mut hmac = <Hmac<Sha1> as KeyInit>::new_from_slice(&password_hash_hash).unwrap();
    hmac.update(nt_response);                    // NT-Response (24 bytes)
    hmac.update(magic1);                         // Magic1 constant
    hmac.update(&challenge_hash[0..8]);          // First 8 bytes of ChallengeHash
    hmac.update(magic2);                         // Magic2 constant
    let authenticator_response = hmac.finalize().into_bytes();

    // Return single 20-byte authenticator response
    authenticator_response.to_vec()
}

#[derive(Debug, Clone)]
pub struct EapPacket {
    pub code: u8,
    pub identifier: u8,
    pub length: u16,
    pub type_: u8,
    pub data: Vec<u8>,
}

impl EapPacket {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let code = data[0];
        let identifier = data[1];
        let length = u16::from_be_bytes([data[2], data[3]]);

        if data.len() < length as usize {
            return None;
        }

        let type_ = if data.len() > 4 { data[4] } else { 0 };
        let data = if data.len() > 5 {
            data[5..length as usize].to_vec()
        } else {
            Vec::new()
        };

        Some(Self {
            code,
            identifier,
            length,
            type_,
            data,
        })
    }


    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(5 + self.data.len());
        out.push(self.code);
        out.push(self.identifier);
        out.extend_from_slice(&self.length.to_be_bytes());
        out.push(self.type_);
        out.extend_from_slice(&self.data);
        out
    }
}

#[derive(Debug, Clone)]
pub struct EapSimAttributes {
    pub version_list: Option<Vec<u8>>,
    pub selected_version: Option<u8>,
    pub nonce_mt: Option<Vec<u8>>,
    pub nonce_s: Option<Vec<u8>>,
    pub rand: Option<Vec<Vec<u8>>>,  // Up to 3 RAND values
    pub mac: Option<Vec<u8>>,
    pub encr_data: Option<Vec<u8>>,
    pub iv: Option<Vec<u8>>,
    pub next_pseudonym: Option<Vec<u8>>,
    pub next_reauth_id: Option<Vec<u8>>,
    pub result_ind: Option<bool>,
    pub counter: Option<u16>,
    pub counter_too_small: Option<bool>,
    pub notification: Option<u16>,
    pub client_error_code: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct EapAkaAttributes {
    pub rand: Option<Vec<u8>>,
    pub autn: Option<Vec<u8>>,
    pub ik: Option<Vec<u8>>,
    pub ck: Option<Vec<u8>>,
    pub res: Option<Vec<u8>>,
    pub auts: Option<Vec<u8>>,
    pub next_pseudonym: Option<Vec<u8>>,
    pub next_reauth_id: Option<Vec<u8>>,
    pub result_ind: Option<bool>,
    pub counter: Option<u16>,
    pub counter_too_small: Option<bool>,
    pub notification: Option<u16>,
    pub client_error_code: Option<u16>,
}
