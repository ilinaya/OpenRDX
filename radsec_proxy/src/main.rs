use std::env;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use dotenv::dotenv;
use tracing::{info, error};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

mod radius;
use radius::{RadiusPacket, forward_packet};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv().ok();
    
    // Get configuration from environment
    let bind_addr = env::var("RADSEC_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:2083".to_string());
    let radius_server = env::var("RADIUS_SERVER").expect("RADIUS_SERVER must be set");
    let radius_secret = env::var("RADIUS_SECRET").expect("RADIUS_SECRET must be set");
    
    // Load TLS certificates
    let cert_base64 = env::var("RADSEC_CERT_BASE64").expect("RADSEC_CERT_BASE64 must be set");
    let key_base64 = env::var("RADSEC_KEY_BASE64").expect("RADSEC_KEY_BASE64 must be set");
    
    let cert = STANDARD.decode(cert_base64)?;
    let key = STANDARD.decode(key_base64)?;
    
    // Configure TLS
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(
            vec![Certificate(cert)],
            PrivateKey(key),
        )?;
    
    let acceptor = TlsAcceptor::from(Arc::new(config));
    
    // Start listening
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("RadSec proxy listening on {}", bind_addr);
    
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("New connection from {}", addr);
        
        let acceptor = acceptor.clone();
        let radius_server = radius_server.clone();
        let radius_secret = radius_secret.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, acceptor, radius_server, radius_secret).await {
                error!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    acceptor: TlsAcceptor,
    radius_server: String,
    radius_secret: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut tls_stream = acceptor.accept(stream).await?;
    
    // Read the RadSec packet
    let mut buf = vec![0u8; 4096];
    let n = tls_stream.read(&mut buf).await?;
    
    if n == 0 {
        return Ok(());
    }
    
    // Parse the RADIUS packet
    if let Some(packet) = RadiusPacket::from_bytes(&buf[..n]) {
        // Forward the packet to the RADIUS server
        match forward_packet(&packet, &radius_server, &radius_secret).await {
            Ok(response) => {
                // Send the response back to the client
                tls_stream.write_all(&response).await?;
            }
            Err(e) => {
                error!("Error forwarding packet: {}", e);
            }
        }
    } else {
        error!("Invalid RADIUS packet received");
    }
    
    Ok(())
}