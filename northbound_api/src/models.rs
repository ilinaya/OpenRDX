use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub api_key_id: Option<u64>,
    pub api_key_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub is_active: bool,
    pub external_id: Option<String>,
    pub allow_any_nas: Option<bool>,
    pub group_ids: Vec<u64>,
    pub identifiers: Vec<UserIdentifier>,
    pub created_at: String,
    pub updated_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserIdentifier {
    pub id: u64,
    pub identifier_type_id: u64,
    pub identifier_type_name: Option<String>,
    pub value: String,
    pub is_enabled: bool,
    pub comment: Option<String>,
    pub auth_attribute_group_id: Option<u64>,
    pub expiration_date: Option<String>,
    pub reject_expired: bool,
    pub expired_auth_attribute_group_id: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreate {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub external_id: Option<String>,
    pub is_active: Option<bool>,
    pub allow_any_nas: Option<bool>,
    pub group_ids: Option<Vec<u64>>,
    pub identifiers: Option<Vec<UserIdentifierCreate>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserIdentifierCreate {
    pub identifier_type_id: u64,
    pub value: String,
    pub plain_password: Option<String>,
    pub is_enabled: Option<bool>,
    pub comment: Option<String>,
    pub auth_attribute_group_id: Option<u64>,
    pub expiration_date: Option<String>,
    pub reject_expired: Option<bool>,
    pub expired_auth_attribute_group_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserIdentifierUpdate {
    pub identifier_type_id: Option<u64>,
    pub value: Option<String>,
    pub plain_password: Option<String>,
    pub is_enabled: Option<bool>,
    pub comment: Option<String>,
    pub auth_attribute_group_id: Option<u64>,
    pub expiration_date: Option<String>,
    pub reject_expired: Option<bool>,
    pub expired_auth_attribute_group_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserIdentifierType {
    pub id: u64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub external_id: Option<String>,
    pub is_active: Option<bool>,
    pub allow_any_nas: Option<bool>,
    pub group_ids: Option<Vec<u64>>,
    pub identifiers: Option<Vec<UserIdentifierCreate>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserListResponse {
    pub count: u64,
    pub results: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserGroup {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<u64>,
    pub allow_any_nas: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserGroupCreate {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<u64>,
    pub allow_any_nas: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserGroupUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<u64>,
    pub allow_any_nas: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NasGroup {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NasGroupCreate {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NasGroupUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Nas {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub ip_address: String,
    pub coa_enabled: bool,
    pub coa_port: u16,
    pub vendor_id: Option<u64>,
    pub secret_id: Option<u64>,
    pub timezone_id: Option<u64>,
    pub group_ids: Vec<u64>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NasCreate {
    pub name: String,
    pub description: Option<String>,
    pub ip_address: String,
    pub coa_enabled: Option<bool>,
    pub coa_port: Option<u16>,
    pub vendor_id: u64,
    pub secret_id: u64,
    pub timezone_id: u64,
    pub group_ids: Option<Vec<u64>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NasUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub ip_address: Option<String>,
    pub coa_enabled: Option<bool>,
    pub coa_port: Option<u16>,
    pub vendor_id: Option<u64>,
    pub secret_id: Option<u64>,
    pub timezone_id: Option<u64>,
    pub group_ids: Option<Vec<u64>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NasListResponse {
    pub count: u64,
    pub results: Vec<Nas>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Vendor {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub vendor_id: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Secret {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
