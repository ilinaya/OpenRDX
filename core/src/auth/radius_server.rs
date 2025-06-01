use tokio::net::UdpSocket;
use tracing::{info, debug, error};
use std::net::IpAddr;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use crate::auth::AuthServer;
use hmac::{Hmac, Mac};
use md5::{Md5};

use digest::{Digest, CtOutput, KeyInit, core_api::CoreWrapper};
use sqlx::PgPool;
use rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use rustls_pemfile::{certs, pkcs8_private_keys};

use des::Des;
use des::cipher::{BlockEncrypt};
use generic_array::GenericArray;

const ATTR_USER_PASSWORD: u8 = 2;      // PAP
const ATTR_CHAP_PASSWORD: u8 = 3;      // CHAP
const VENDOR_MICROSOFT: u32 = 311;       // Microsoft's Vendor-ID

const VENDOR_ATTR_MS_CHAP_CHALLENGE: u8 = 11;  // Microsoft's MS-CHAP-Challenge
const VENDOR_ATTR_MS_CHAP_RESPONSE: u8 = 1;    // Microsoft's MS-CHAP-Response

const ATTR_VENDOR_SPECIFIC: u8 = 26;   // VSA attribute type

const ATTR_MS_CHAP2_RESPONSE: u8 = 25; // MS-CHAPv2

const VENDOR_ATTR_MS_CHAP2_RESPONSE: u8 = 25;    // Microsoft's MS-CHAPv2-Response
const VENDOR_ATTR_MS_CHAP2_CHALLENGE: u8 = 11;   // Microsoft's MS-CHAPv2-Challenge

const ATTR_USER_NAME: u8 = 1;

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
        // Check for EAP first
        if packet.attributes.iter().any(|attr| attr.typ == ATTR_EAP_MESSAGE) {
            return "EAP".to_string();
        }

        let has_mschap = packet.attributes.iter().any(|attr| {
            if attr.typ == ATTR_VENDOR_SPECIFIC && attr.value.len() >= 4 {
                let vendor_id = u32::from_be_bytes([
                    attr.value[0], attr.value[1],
                    attr.value[2], attr.value[3]
                ]);
                if vendor_id == VENDOR_MICROSOFT && attr.value.len() >= 6 {
                    let vendor_type = attr.value[4];
                    return vendor_type == VENDOR_ATTR_MS_CHAP_RESPONSE;
                }
            }
            false
        });

        if has_mschap {
            return "MS-CHAP".to_string();
        }

        if packet.attributes.iter().any(|attr| attr.typ == ATTR_USER_PASSWORD) {
            "PAP".to_string()
        } else if packet.attributes.iter().any(|attr| attr.typ == ATTR_CHAP_PASSWORD) {
            "CHAP".to_string()
        } else if packet.attributes.iter().any(|attr| attr.typ == ATTR_MS_CHAP2_RESPONSE) {
            "MS-CHAPv2".to_string()
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
                                    let response = self.handle_access_request(&packet, secret).await;
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

    async fn handle_access_request(&self, packet: &RadiusPacket, secret: &str) -> Vec<u8> {
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
                        let vendor_length = attr.value[5] as usize;
                        let vendor_data = &attr.value[6..];

                        match vendor_type {
                            VENDOR_ATTR_MS_CHAP_CHALLENGE => {
                                debug!("Found MS-CHAP-Challenge in VSA");
                                mschap_challenge = Some(vendor_data.to_vec());
                            }
                            VENDOR_ATTR_MS_CHAP_RESPONSE => {
                                debug!("Found MS-CHAP-Response in VSA");
                                if vendor_data.len() >= 49 {
                                    mschap_response = Some(vendor_data[26..50].to_vec());
                                }
                            }
                            VENDOR_ATTR_MS_CHAP2_RESPONSE => {
                                debug!("Found MS-CHAPv2-Response in VSA");
                                if vendor_data.len() >= 50 {
                                    mschap2_peer_challenge = Some(vendor_data[2..18].to_vec());
                                    mschap2_nt_response = Some(vendor_data[26..50].to_vec());
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
                debug!("Username: {:?}", username);
                debug!("Challenge present: {}", mschap_challenge.is_some());
                debug!("Response present: {}", mschap_response.is_some());
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
                if let (Some(username), Some(peer_challenge), Some(nt_response)) = (username, mschap2_peer_challenge, mschap2_nt_response) {
                    match self.authenticate_mschap2(&username, &peer_challenge, &nt_response, &packet.authenticator, secret).await {
                        Ok(AuthResult::Success) => self.create_access_accept(packet, secret),
                        Ok(AuthResult::UserNotFound) => self.create_access_reject(packet, secret, "User not found"),
                        Ok(AuthResult::InvalidPassword) => self.create_access_reject(packet, secret, "Invalid MS-CHAPv2 response"),
                        Ok(AuthResult::AccountDisabled) => self.create_access_reject(packet, secret, "Account is disabled"),
                        Ok(AuthResult::DatabaseError(_)) => self.create_access_reject(packet, secret, "Internal server error"),
                        Err(_) => self.create_access_reject(packet, secret, "Internal server error"),
                    }
                } else {
                    self.create_access_reject(packet, secret, "Invalid MS-CHAPv2 request")
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
                        debug!("MS-CHAP response mismatch");
                        debug!("Expected: {:?}", challenge_response);
                        debug!("Received: {:?}", response);
                        Ok(AuthResult::InvalidPassword)
                    }
                } else {
                    Ok(AuthResult::InvalidPassword)
                }
            }
            None => Ok(AuthResult::UserNotFound),
        }
    }


    async fn authenticate_mschap2(&self, username: &str, peer_challenge: &[u8], nt_response: &[u8], authenticator: &[u8], secret: &str) -> Result<AuthResult, sqlx::Error> {
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
                    // Generate NT hash from password
                    let nt_hash = nt_hash(stored_pass.as_bytes());
                    
                    // Generate challenge hash using SHA1(peer_challenge + authenticator + username)
                    let mut hasher = sha1::Sha1::new();
                    hasher.update(peer_challenge);
                    hasher.update(authenticator);
                    hasher.update(username.as_bytes());
                    let challenge_hash = hasher.finalize();

                    // Generate expected NT response
                    let mut hasher = md4::Md4::new();
                    hasher.update(&nt_hash);
                    hasher.update(&challenge_hash);

                    let challenge_hash = {
                        let mut sha = sha1::Sha1::new();
                        sha.update(peer_challenge);
                        sha.update(authenticator);
                        sha.update(username.as_bytes());
                        let full_hash = sha.finalize();
                        full_hash[0..8].to_vec()
                    };

                    // Generate 24-byte response with 3 DES blocks
                    let mut padded = nt_hash.clone();
                    padded.resize(21, 0);

                    let mut expected_response = Vec::with_capacity(24);

                    for i in 0..3 {
                        let key_7 = &padded[i * 7..(i + 1) * 7];
                        let key_8 = setup_des_key(key_7);
                        let cipher = Des::new_from_slice(&key_8).unwrap();
                        let mut block = GenericArray::clone_from_slice(&challenge_hash[..8]);
                        cipher.encrypt_block(&mut block);
                        expected_response.extend_from_slice(&block);
                    }

                    // Compare NT responses (24 bytes)
                    if nt_response[0..24] == expected_response.as_slice()[0..24] {
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
            let mut mac = <Hmac<Md5> as Mac>::new_from_slice(secret.as_bytes())
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
        let mut sim_attrs = EapSimAttributes {
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
                let mut response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_SIM,
                    data: vec![EAP_SIM_START], // Start subtype
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
        let mut aka_attrs = EapAkaAttributes {
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
                let mut response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_AKA,
                    data: vec![EAP_AKA_IDENTITY], // Identity subtype
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
                let mut response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_TLS,
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
                let mut response = EapPacket {
                    code: EAP_RESPONSE,
                    identifier: eap_packet.identifier,
                    length: 0, // Will be set after encoding
                    type_: EAP_TYPE_PEAP,
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

        let acceptor = TlsAcceptor::from(Arc::new(config));

        match eap_packet.code {
            EAP_REQUEST => {
                // Start TTLS handshake
                let mut response = EapPacket {
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

// Helper function for MS-CHAPv2
fn nt_hash(password: &[u8]) -> Vec<u8> {
    let mut hasher = md4::Md4::new();
    hasher.update(password);
    hasher.finalize().to_vec()
}

/// Convert 7-byte array into 8-byte DES key (with parity bits)
fn setup_des_key(key_7: &[u8]) -> [u8; 8] {
    let mut key = [0u8; 8];
    key[0] = key_7[0];
    key[1] = ((key_7[0] << 7) | (key_7[1] >> 1));
    key[2] = ((key_7[1] << 6) | (key_7[2] >> 2));
    key[3] = ((key_7[2] << 5) | (key_7[3] >> 3));
    key[4] = ((key_7[3] << 4) | (key_7[4] >> 4));
    key[5] = ((key_7[4] << 3) | (key_7[5] >> 5));
    key[6] = ((key_7[5] << 2) | (key_7[6] >> 6));
    key[7] = (key_7[6] << 1);
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