use async_std::net::UdpSocket;
use async_std::task;
use async_trait::async_trait;
use dotenv::dotenv;
use futures::{future::FutureExt, pin_mut, select};
use log::{debug, LevelFilter};
use radius_rust::protocol::dictionary::Dictionary;
use radius_rust::protocol::error::RadiusError;
use radius_rust::protocol::radius_packet::{RadiusMsgType, TypeCode, RadiusPacket};
use radius_rust::server::{server::Server, AsyncServerTrait};
use radius_rust::tools::{integer_to_bytes, ipv4_string_to_bytes};
use simple_logger::SimpleLogger;
use std::env;
use md4::{Md4, Digest};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use hex;

// Hardcoded user credentials for PAP and MSCHAPv2
const USERS: &[(&str, &str)] = &[
    ("test_user", "password123"),
    ("user2", "pass456"),
];

// EAP-AKA SIM credentials
const IMSI: &str = "001017890123453";
const OP: &str = "9B7DF5E24D2B89C91E39086BEFEC259F";
const KI: &str = "2E7DDA5E369011EFC380B17CB6CDB4C5";

/// Extend the server with custom configurations and request handlers
struct CustomServer {
    base_server: Server,
    auth_socket: UdpSocket,
    acct_socket: UdpSocket,
    coa_socket: UdpSocket,
    dictionary: Dictionary,
    dictionary_path: String,  // Store the path separately
}

impl CustomServer {
    /// Initialize the server by parsing configurations and creating sockets
    async fn initialise_server(
        auth_port: u16,
        acct_port: u16,
        coa_port: u16,
        dictionary: Dictionary,
        dictionary_path: String,  // Add dictionary_path parameter
        server_addr: String,
        secret: String,
        retries: u16,
        timeout: u16,
        allowed_hosts: Vec<String>,
    ) -> Result<CustomServer, RadiusError> {
        // Initialise sockets
        let auth_socket = UdpSocket::bind(format!("{}:{}", &server_addr, auth_port)).await?;
        let acct_socket = UdpSocket::bind(format!("{}:{}", &server_addr, acct_port)).await?;
        let coa_socket = UdpSocket::bind(format!("{}:{}", &server_addr, coa_port)).await?;

        debug!(
            "Authentication Server is started on {}",
            &auth_socket.local_addr()?
        );
        debug!(
            "Accounting Server is started on {}",
            &acct_socket.local_addr()?
        );
        debug!(
            "CoA Server is started on {}",
            &coa_socket.local_addr()?
        );

        // Create a new dictionary instance for the server
        let server_dictionary = Dictionary::from_file(&dictionary_path)?;
        
        let server = Server::with_dictionary(server_dictionary)
            .set_server(server_addr)
            .set_secret(secret)
            .set_port(RadiusMsgType::AUTH, auth_port)
            .set_port(RadiusMsgType::ACCT, acct_port)
            .set_port(RadiusMsgType::COA, coa_port)
            .set_allowed_hosts(allowed_hosts)
            .set_retries(retries)
            .set_timeout(timeout);

        Ok(CustomServer {
            base_server: server,
            auth_socket,
            acct_socket,
            coa_socket,
            dictionary,
            dictionary_path,
        })
    }

    /// Authenticates using PAP
    fn authenticate_pap(&self, username: &str, password: &str) -> bool {
        USERS.iter().any(|(u, p)| u == &username && p == &password)
    }

    /// Authenticates using MSCHAPv2
    fn authenticate_mschapv2(&self, username: &str, challenge: &[u8], response: &[u8]) -> bool {
        // Find user's password
        let password = USERS.iter()
            .find(|(u, _)| u == &username)
            .map(|(_, p)| p.as_bytes())
            .unwrap_or(&[]);

        if password.is_empty() {
            return false;
        }

        // Generate NT hash of password
        let mut nt_hash = [0u8; 16];
        let mut hasher = Md4::new();
        hasher.update(password);
        nt_hash.copy_from_slice(&hasher.finalize());

        // Generate challenge response
        let mut challenge_response = [0u8; 24];
        let mut hmac = Hmac::<Sha1>::new_from_slice(&nt_hash).unwrap();
        hmac.update(challenge);
        hmac.update(&[0u8; 8]); // Peer challenge
        hmac.update(&[0u8; 8]); // Reserved
        challenge_response.copy_from_slice(&hmac.finalize().into_bytes()[..24]);

        // Compare with received response
        challenge_response == response
    }

    /// Helper function to convert hex string to bytes
    fn hex_to_bytes(hex: &str) -> Vec<u8> {
        hex::decode(hex).unwrap_or_default()
    }

    /// Helper function to generate OPc from OP and KI
    fn generate_opc(&self) -> Vec<u8> {
        // In a real implementation, this would use the Milenage algorithm
        // For now, we'll just return a dummy value
        CustomServer::hex_to_bytes(OP)
    }

    /// Authenticates using EAP-AKA
    fn authenticate_eap_aka(&self, identity: &str, challenge: &[u8], response: &[u8]) -> bool {
        debug!("=== EAP-AKA Authentication Attempt ===");
        debug!("Received identity: {}", identity);
        debug!("Expected IMSI: {}", IMSI);
        debug!("Challenge length: {} bytes", challenge.len());
        debug!("Response length: {} bytes", response.len());
        
        // Check if the identity matches our IMSI
        if identity != IMSI {
            debug!("❌ IMSI mismatch: expected {}, got {}", IMSI, identity);
            return false;
        }
        debug!("✅ IMSI match: {}", identity);

        // Get the OPc value
        let opc = self.generate_opc();
        let ki = CustomServer::hex_to_bytes(KI);

        debug!("Using OP: {}", OP);
        debug!("Using OPc: {}", hex::encode(&opc));
        debug!("Using KI: {}", hex::encode(&ki));
        debug!("Challenge: {}", hex::encode(challenge));
        debug!("Response: {}", hex::encode(response));

        // In a real implementation, we would:
        // 1. Generate AUTN from OPc, KI, and challenge
        // 2. Verify client's response
        // 3. Generate RES and compare with XRES
        
        debug!("✅ EAP-AKA Authentication successful for IMSI: {}", identity);
        true
    }

    fn get_auth_method(&self, request_data: &[u8]) -> Option<&str> {
        // Log the raw RADIUS request data for debugging
        debug!("Raw request data ({} bytes): {:?}", request_data.len(), hex::encode(request_data));

        // Parse the RADIUS packet
        match RadiusPacket::initialise_packet_from_bytes(&self.dictionary, request_data) {
            Ok(packet) => {
                debug!("Successfully parsed RADIUS packet: {:?}", packet);

                // Check for EAP-Message attribute (EAP-AKA)
                debug!("Checking for EAP-Message attribute...");
                if let Some(eap_message) = packet.attribute_by_name("EAP-Message") {
                    debug!("Found EAP-Message attribute: {:?}", hex::encode(eap_message.value()));
                    // Check if it's an EAP-AKA request (EAP code 1 = Request, Type 23 = AKA)
                    return Some("EAP-AKA");
                }

                // Check for MS-CHAP attributes
                debug!("Checking for MS-CHAP attributes...");
                if packet.attribute_by_name("MS-CHAP-Challenge").is_some() {
                    return Some("MSCHAPv2");
                }

                // If we have User-Password, it's PAP
                debug!("Checking for User-Password attribute...");
                if packet.attribute_by_name("User-Password").is_some() {
                    return Some("PAP");
                }

                debug!("Authentication method not recognized in parsed packet");
            }
            Err(e) => {
                // Log the error for debugging purposes
                debug!("Failed to parse RADIUS packet: {:?}", e);
            }
        }

        debug!("Could not determine authentication method from packet");
        None
    }

}

#[async_trait]
impl AsyncServerTrait for CustomServer {
    

    
    async fn run(&mut self) -> Result<(), RadiusError> {
        let auth_task = self.handle_auth_request().fuse();
        let acct_task = self.handle_acct_request().fuse();
        let coa_task = self.handle_coa_request().fuse();

        pin_mut!(auth_task, acct_task, coa_task);

        select! {
            _ = auth_task => Ok(()),
            _ = acct_task => Ok(()),
            _ = coa_task => Ok(()),
        }
    }

    async fn handle_auth_request(&self) -> Result<(), RadiusError> {
        loop {
            debug!("Handling AUTH request");

            // Read RADIUS packet
            let mut request = [0u8; 4096];
            let (size, source_addr) = self.auth_socket.recv_from(&mut request).await?;
            let request_data = &request[..size];
            debug!("Received AUTH request from {}", source_addr);

            let auth_method = self.get_auth_method(request_data);
            let mut success = false;
            let mut username = String::new();

            match auth_method {
                Some("PAP") => {
                    // Extract username and password from PAP request
                    if let Ok(packet) = RadiusPacket::initialise_packet_from_bytes(&self.dictionary, request_data) {
                        if let Some(user_name) = packet.attribute_by_name("User-Name") {
                            username = String::from_utf8_lossy(user_name.value()).to_string();
                        }
                        if let Some(password) = packet.attribute_by_name("User-Password") {
                            success = self.authenticate_pap(&username, &String::from_utf8_lossy(password.value()));
                        }
                    }
                }
                Some("MSCHAPv2") => {
                    // Extract MSCHAPv2 challenge and response
                    if let Ok(packet) = RadiusPacket::initialise_packet_from_bytes(&self.dictionary, request_data) {
                        if let Some(user_name) = packet.attribute_by_name("User-Name") {
                            username = String::from_utf8_lossy(user_name.value()).to_string();
                        }
                        if let Some(challenge) = packet.attribute_by_name("MS-CHAP-Challenge") {
                            if let Some(response) = packet.attribute_by_name("MS-CHAP-Response") {
                                success = self.authenticate_mschapv2(&username, challenge.value(), response.value());
                            }
                        }
                    }
                }
                Some("EAP-AKA") => {
                    // Extract EAP-AKA identity and response
                    if let Ok(packet) = RadiusPacket::initialise_packet_from_bytes(&self.dictionary, request_data) {
                        if let Some(eap_message) = packet.attribute_by_name("EAP-Message") {
                            let eap_data = eap_message.value();
                            debug!("EAP message data: {:?}", hex::encode(eap_data));
                            
                            // Check if this is an identity request or response to our challenge
                            if eap_data.len() >= 5 && eap_data[4] == 1 {  // Type 1 = Identity
                                // Handle identity request
                                if eap_data.len() >= 6 {
                                    let identity_length = eap_data[5] as usize;
                                    if eap_data.len() >= 6 + identity_length {
                                        username = String::from_utf8_lossy(&eap_data[6..6+identity_length]).to_string();
                                        debug!("Extracted EAP-AKA identity: {}", username);
                                        
                                        // Generate EAP-AKA challenge
                                        let mut eap_challenge = vec![
                                            1,  // Code: Request
                                            eap_data[1],  // Use same identifier
                                            0, 0,  // Length will be set later
                                            23,  // Type: EAP-AKA
                                            1,  // Subtype: Challenge
                                            0, 0, 0, 0, 0, 0, 0, 0,  // RAND
                                            0, 0, 0, 0, 0, 0, 0, 0,  // AUTN
                                            0, 0, 0, 0, 0, 0, 0, 0,  // MAC
                                        ];
                                        
                                        // Set length (2 bytes, network byte order)
                                        let length = eap_challenge.len() as u16;
                                        eap_challenge[2] = (length >> 8) as u8;
                                        eap_challenge[3] = length as u8;
                                        
                                        // Create Access-Challenge response
                                        let attributes = vec![
                                            self.base_server.create_attribute_by_name("EAP-Message", eap_challenge)?,
                                            self.base_server.create_attribute_by_name("State", vec![0x01, 0x02, 0x03, 0x04])?,
                                        ];
                                        
                                        let mut reply_packet = self
                                            .base_server
                                            .create_reply_packet(TypeCode::AccessChallenge, attributes, &mut request.clone());
                                        self.auth_socket.send_to(&reply_packet.to_bytes(), &source_addr).await?;
                                        continue;  // Skip the success/failure response below
                                    }
                                }
                            } else if eap_data.len() >= 5 && eap_data[4] == 23 {  // Type 23 = EAP-AKA
                                // Handle EAP-AKA response to our challenge
                                debug!("Received EAP-AKA response to challenge");

                                if eap_data.len() >= 6 && eap_data[5] == 2 {  // Subtype 2 = Response
                                    debug!("Received EAP-AKA response to challenge");
                                    
                                    // For now, we'll accept any response
                                    // In a real implementation, we would verify the response
                                    let mut eap_success = vec![
                                        3,  // Code: Success
                                        eap_data[1],  // Use same identifier
                                        0, 0,  // Length will be set later
                                    ];
                                    
                                    // Set length (2 bytes, network byte order)
                                    let length = eap_success.len() as u16;
                                    eap_success[2] = (length >> 8) as u8;
                                    eap_success[3] = length as u8;
                                    
                                    // Create Access-Accept response
                                    let attributes = vec![
                                        self.base_server.create_attribute_by_name("EAP-Message", eap_success)?,
                                        self.base_server.create_attribute_by_name("Service-Type", integer_to_bytes(2))?,
                                        self.base_server.create_attribute_by_name("Framed-IP-Address", ipv4_string_to_bytes("192.168.0.100")?)?,
                                    ];
                                    
                                    let mut reply_packet = self
                                        .base_server
                                        .create_reply_packet(TypeCode::AccessAccept, attributes, &mut request.clone());
                                    self.auth_socket.send_to(&reply_packet.to_bytes(), &source_addr).await?;
                                    continue;  // Skip the success/failure response below
                                }
                            }
                            
                            debug!("Unhandled EAP message type or format");
                        }
                    }
                }
                Some(_) => {
                    debug!("Unsupported authentication method");
                }
                None => {
                    debug!("Unknown authentication method");
                }
            }

            let attributes = if success {
                debug!("Authentication successful for user: {}", username);
                vec![
                    self.base_server
                        .create_attribute_by_name("Service-Type", integer_to_bytes(2))?,
                    self.base_server
                        .create_attribute_by_name("Framed-IP-Address", ipv4_string_to_bytes("192.168.0.100")?)?,
                ]
            } else {
                debug!("Authentication failed for user: {}", username);
                vec![]
            };

            let reply_type = if success {
                TypeCode::AccessAccept
            } else {
                TypeCode::AccessReject
            };

            let mut reply_packet = self
                .base_server
                .create_reply_packet(reply_type, attributes, &mut request.clone());
            self.auth_socket.send_to(&reply_packet.to_bytes(), &source_addr).await?;
        }
    }

    async fn handle_acct_request(&self) -> Result<(), RadiusError> {
        loop {
            debug!("Handling ACCT request");
            // For simplicity, this handler just echoes back an AccountingResponse
            let mut request = [0u8; 4096];
            let (size, source_addr) = self.acct_socket.recv_from(&mut request).await?;
            let _request_data = &request[..size];

            let attributes = vec![];
            let mut reply_packet = self
                .base_server
                .create_reply_packet(TypeCode::AccountingResponse, attributes, &mut request.clone());
            self.acct_socket.send_to(&reply_packet.to_bytes(), &source_addr).await?;
        }
    }

    async fn handle_coa_request(&self) -> Result<(), RadiusError> {
        loop {
            debug!("Handling CoA request");

            // Placeholder response for CoA requests
            let mut request = [0u8; 4096];
            let (size, source_addr) = self.coa_socket.recv_from(&mut request).await?;
            let _request_data = &request[..size];

            let attributes = vec![];
            let mut reply_packet = self
                .base_server
                .create_reply_packet(TypeCode::CoAACK, attributes, &mut request.clone());
            self.coa_socket.send_to(&reply_packet.to_bytes(), &source_addr).await?;
        }
    }
}

fn main() -> Result<(), RadiusError> {
    dotenv().ok(); // Load .env configuration

    // Load server configurations from .env
    let server_addr = env::var("LISTEN_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let auth_port = env::var("LISTEN_AUTH_PORT").unwrap_or_else(|_| "1812".to_string());
    let acct_port = env::var("LISTEN_ACCT_PORT").unwrap_or_else(|_| "1813".to_string());

    let secret = env::var("RADIUS_SECRET").expect("RADIUS_SECRET not found in .env!");
    let dictionary_path = env::var("RADIUS_DICTIONARY").unwrap_or_else(|_| "./dict_examples/integration_dict".to_string());



    task::block_on(async {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .unwrap();

        debug!("Async RADIUS Server starting...");

        let dictionary = Dictionary::from_file(&*dictionary_path)
            .expect("Failed to load RADIUS dictionary");
        let allowed_hosts = vec![server_addr.clone()];
        let mut server = CustomServer::initialise_server(
            auth_port.parse().unwrap(),
            acct_port.parse().unwrap(),
            3799,
            dictionary,
            dictionary_path,
            server_addr,
            secret,
            3, // retries
            5, // timeout
            allowed_hosts,
        )
            .await
            .expect("Failed to initialize RADIUS server");

        server.run().await
    })
}