use serde::{Serialize, Deserialize};
use std::net::IpAddr;
use ipnetwork::IpNetwork;
use sqlx::types::JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscriber {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub group_id: Option<i32>,
    pub is_active: bool,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NasDevice {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Secret {
    pub id: i32,
    pub source_subnets: Option<JsonValue>,
    pub secret: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthAttribute {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthAttributeGroup {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub attributes: Vec<AuthAttribute>,
} 