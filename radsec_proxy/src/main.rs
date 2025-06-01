use dotenv::dotenv;
use std::path::Path;
use tracing::{info, error, Level, debug};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan, FmtSubscriber};
use std::sync::Arc;

mod auth;
mod accounting;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging with more detailed configuration
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    Ok(())
}