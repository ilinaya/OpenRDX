use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub radius_bind_addr: String,
    pub radius_secret_ttl: u64,
    pub subscriber_cache_ttl: u64,
    pub log_level: String,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")?,
            radius_bind_addr: std::env::var("RADIUS_BIND_ADDR")?,
            radius_secret_ttl: std::env::var("RADIUS_SECRET_TTL")?.parse()?,
            subscriber_cache_ttl: std::env::var("SUBSCRIBER_CACHE_TTL")?.parse()?,
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://user:pass@localhost/db".to_string(),
            redis_url: "redis://127.0.0.1/".to_string(),
            radius_bind_addr: "127.0.0.1:1812".to_string(),
            radius_secret_ttl: 300,
            subscriber_cache_ttl: 3600,
            log_level: "info".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let config = r#"{
            "database_url": "postgres://test:test@localhost/test",
            "redis_url": "redis://127.0.0.1/",
            "radius_bind_addr": "127.0.0.1:1812",
            "radius_secret_ttl": 300,
            "subscriber_cache_ttl": 3600,
            "log_level": "debug"
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, config).unwrap();

        let loaded_config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(loaded_config.database_url, "postgres://test:test@localhost/test");
        assert_eq!(loaded_config.redis_url, "redis://127.0.0.1/");
        assert_eq!(loaded_config.radius_bind_addr, "127.0.0.1:1812");
        assert_eq!(loaded_config.radius_secret_ttl, 300);
        assert_eq!(loaded_config.subscriber_cache_ttl, 3600);
        assert_eq!(loaded_config.log_level, "debug");
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.database_url, "postgres://user:pass@localhost/db");
        assert_eq!(config.redis_url, "redis://127.0.0.1/");
        assert_eq!(config.radius_bind_addr, "127.0.0.1:1812");
        assert_eq!(config.radius_secret_ttl, 300);
        assert_eq!(config.subscriber_cache_ttl, 3600);
        assert_eq!(config.log_level, "info");
    }
} 