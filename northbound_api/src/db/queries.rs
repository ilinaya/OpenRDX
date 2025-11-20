use deadpool_postgres::Pool;
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;
use std::time::SystemTime;
use anyhow::Result;

// User queries
pub async fn list_users(pool: &Pool, page: i64, page_size: i64) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let offset = (page - 1) * page_size;
    
    let rows = client.query(
        "SELECT id, email, first_name, last_name, phone_number, is_active, external_id, allow_any_nas, created_at, updated_at, last_login 
         FROM users 
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2",
        &[&page_size, &offset],
    ).await?;
    
    Ok(rows)
}

pub async fn get_user(pool: &Pool, id: i64) -> Result<Option<Row>> {
    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id, email, first_name, last_name, phone_number, is_active, external_id, allow_any_nas, created_at, updated_at, last_login 
         FROM users 
         WHERE id = $1",
        &[&id],
    ).await?;
    
    Ok(row)
}

pub async fn create_user(
    pool: &Pool,
    email: &str,
    first_name: Option<&str>,
    last_name: Option<&str>,
    phone_number: Option<&str>,
    external_id: Option<&str>,
    is_active: bool,
    allow_any_nas: Option<bool>,
) -> Result<i64> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    let row = client.query_one(
        "INSERT INTO users (email, first_name, last_name, phone_number, external_id, is_active, allow_any_nas, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING id",
        &[&email, &first_name, &last_name, &phone_number, &external_id, &is_active, &allow_any_nas, &now, &now],
    ).await?;
    
    Ok(row.get(0))
}

pub async fn update_user(
    pool: &Pool,
    id: i64,
    email: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
    phone_number: Option<&str>,
    external_id: Option<&str>,
    is_active: Option<bool>,
    allow_any_nas: Option<bool>,
) -> Result<bool> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    // Build dynamic UPDATE query - two phase approach
    // Phase 1: Collect all string values
    let mut string_params: Vec<String> = Vec::new();
    let mut updates = Vec::new();
    let mut param_index = 1;
    
    if email.is_some() {
        updates.push(format!("email = ${}", param_index));
        string_params.push(email.unwrap().to_string());
        param_index += 1;
    }
    if first_name.is_some() {
        updates.push(format!("first_name = ${}", param_index));
        string_params.push(first_name.unwrap().to_string());
        param_index += 1;
    }
    if last_name.is_some() {
        updates.push(format!("last_name = ${}", param_index));
        string_params.push(last_name.unwrap().to_string());
        param_index += 1;
    }
    if phone_number.is_some() {
        updates.push(format!("phone_number = ${}", param_index));
        string_params.push(phone_number.unwrap().to_string());
        param_index += 1;
    }
    if external_id.is_some() {
        updates.push(format!("external_id = ${}", param_index));
        string_params.push(external_id.unwrap().to_string());
        param_index += 1;
    }
    if is_active.is_some() {
        updates.push(format!("is_active = ${}", param_index));
        param_index += 1;
    }
    if allow_any_nas.is_some() {
        updates.push(format!("allow_any_nas = ${}", param_index));
        param_index += 1;
    }
    
    updates.push(format!("updated_at = ${}", param_index));
    param_index += 1;
    
    // Phase 2: Build params array - all values collected, safe to take references
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
    let mut string_idx = 0;
    
    if email.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if first_name.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if last_name.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if phone_number.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if external_id.is_some() {
        params.push(&string_params[string_idx]);
        // No need to increment - this is the last string parameter
    }
    if let Some(ref val) = is_active {
        params.push(val);
    }
    if let Some(ref val) = allow_any_nas {
        params.push(val);
    }
    params.push(&now);
    params.push(&id);
    
    let query = format!(
        "UPDATE users SET {} WHERE id = ${}",
        updates.join(", "),
        param_index
    );
    
    let result = client.execute(&query, &params).await?;
    Ok(result > 0)
}

pub async fn delete_user(pool: &Pool, id: i64) -> Result<bool> {
    let client = pool.get().await?;
    let result = client.execute("DELETE FROM users WHERE id = $1", &[&id]).await?;
    Ok(result > 0)
}

pub async fn count_users(pool: &Pool) -> Result<i64> {
    let client = pool.get().await?;
    let row = client.query_one("SELECT COUNT(*) FROM users", &[]).await?;
    Ok(row.get(0))
}

// User Groups queries
pub async fn list_user_groups(pool: &Pool) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT id, name, description, parent_id, allow_any_nas, created_at, updated_at 
         FROM user_groups 
         ORDER BY name",
        &[],
    ).await?;
    
    Ok(rows)
}

pub async fn get_user_group(pool: &Pool, id: i64) -> Result<Option<Row>> {
    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id, name, description, parent_id, allow_any_nas, created_at, updated_at 
         FROM user_groups 
         WHERE id = $1",
        &[&id],
    ).await?;
    
    Ok(row)
}

pub async fn create_user_group(
    pool: &Pool,
    name: &str,
    description: Option<&str>,
    parent_id: Option<i64>,
    allow_any_nas: bool,
) -> Result<i64> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    let row = client.query_one(
        "INSERT INTO user_groups (name, description, parent_id, allow_any_nas, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id",
        &[&name, &description, &parent_id, &allow_any_nas, &now, &now],
    ).await?;
    
    Ok(row.get(0))
}

pub async fn update_user_group(
    pool: &Pool,
    id: i64,
    name: Option<&str>,
    description: Option<&str>,
    parent_id: Option<i64>,
    allow_any_nas: Option<bool>,
) -> Result<bool> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    // Two phase approach: collect values first, then build params
    let mut string_params: Vec<String> = Vec::new();
    let mut updates = Vec::new();
    let mut param_index = 1;
    
    if name.is_some() {
        updates.push(format!("name = ${}", param_index));
        string_params.push(name.unwrap().to_string());
        param_index += 1;
    }
    if description.is_some() {
        updates.push(format!("description = ${}", param_index));
        string_params.push(description.unwrap().to_string());
        param_index += 1;
    }
    if parent_id.is_some() {
        updates.push(format!("parent_id = ${}", param_index));
        param_index += 1;
    }
    if allow_any_nas.is_some() {
        updates.push(format!("allow_any_nas = ${}", param_index));
        param_index += 1;
    }
    
    updates.push(format!("updated_at = ${}", param_index));
    param_index += 1;
    
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
    let mut string_idx = 0;
    
    if name.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if description.is_some() {
        params.push(&string_params[string_idx]);
        // No need to increment - this is the last string parameter
    }
    if let Some(ref val) = parent_id {
        params.push(val);
    }
    if let Some(ref val) = allow_any_nas {
        params.push(val);
    }
    params.push(&now);
    params.push(&id);
    
    let query = format!(
        "UPDATE user_groups SET {} WHERE id = ${}",
        updates.join(", "),
        param_index
    );
    
    let result = client.execute(&query, &params).await?;
    Ok(result > 0)
}

pub async fn delete_user_group(pool: &Pool, id: i64) -> Result<bool> {
    let client = pool.get().await?;
    let result = client.execute("DELETE FROM user_groups WHERE id = $1", &[&id]).await?;
    Ok(result > 0)
}

// User-Group associations
pub async fn set_user_groups(pool: &Pool, user_id: i64, group_ids: &[i64]) -> Result<()> {
    let client = pool.get().await?;
    
    // Delete existing associations
    client.execute(
        "DELETE FROM users_groups WHERE user_id = $1",
        &[&user_id],
    ).await?;
    
    // Insert new associations
    for group_id in group_ids {
        client.execute(
            "INSERT INTO users_groups (user_id, usergroup_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            &[&user_id, group_id],
        ).await?;
    }
    
    Ok(())
}

pub async fn get_user_groups(pool: &Pool, user_id: i64) -> Result<Vec<i64>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT usergroup_id FROM users_groups WHERE user_id = $1",
        &[&user_id],
    ).await?;
    
    Ok(rows.iter().map(|row| row.get(0)).collect())
}

// User Identifiers queries
pub async fn create_user_identifier(
    pool: &Pool,
    user_id: i64,
    identifier_type_id: i64,
    value: &str,
    plain_password: Option<&str>,
    is_enabled: bool,
    comment: Option<&str>,
    auth_attribute_group_id: Option<i64>,
    expiration_date: Option<SystemTime>,
    reject_expired: bool,
    expired_auth_attribute_group_id: Option<i64>,
) -> Result<i64> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    let row = client.query_one(
        "INSERT INTO user_identifiers (user_id, identifier_type_id, value, plain_password, is_enabled, comment, auth_attribute_group_id, expiration_date, reject_expired, expired_auth_attribute_group_id, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) 
         RETURNING id",
        &[&user_id, &identifier_type_id, &value, &plain_password, &is_enabled, &comment, &auth_attribute_group_id, &expiration_date, &reject_expired, &expired_auth_attribute_group_id, &now, &now],
    ).await?;
    
    Ok(row.get(0))
}

pub async fn update_user_identifier(
    pool: &Pool,
    id: i64,
    identifier_type_id: Option<i64>,
    value: Option<&str>,
    plain_password: Option<&str>,
    is_enabled: Option<bool>,
    comment: Option<&str>,
    auth_attribute_group_id: Option<i64>,
    expiration_date: Option<Option<SystemTime>>,
    reject_expired: Option<bool>,
    expired_auth_attribute_group_id: Option<i64>,
) -> Result<bool> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    // Two phase approach: collect values first, then build params
    let mut string_params: Vec<String> = Vec::new();
    let mut updates = Vec::new();
    let mut param_index = 1;
    
    if identifier_type_id.is_some() {
        updates.push(format!("identifier_type_id = ${}", param_index));
        param_index += 1;
    }
    if value.is_some() {
        updates.push(format!("value = ${}", param_index));
        string_params.push(value.unwrap().to_string());
        param_index += 1;
    }
    if plain_password.is_some() {
        updates.push(format!("plain_password = ${}", param_index));
        string_params.push(plain_password.unwrap().to_string());
        param_index += 1;
    }
    if is_enabled.is_some() {
        updates.push(format!("is_enabled = ${}", param_index));
        param_index += 1;
    }
    if comment.is_some() {
        updates.push(format!("comment = ${}", param_index));
        string_params.push(comment.unwrap().to_string());
        param_index += 1;
    }
    if auth_attribute_group_id.is_some() {
        updates.push(format!("auth_attribute_group_id = ${}", param_index));
        param_index += 1;
    }
    if expiration_date.is_some() {
        updates.push(format!("expiration_date = ${}", param_index));
        param_index += 1;
    }
    if reject_expired.is_some() {
        updates.push(format!("reject_expired = ${}", param_index));
        param_index += 1;
    }
    if expired_auth_attribute_group_id.is_some() {
        updates.push(format!("expired_auth_attribute_group_id = ${}", param_index));
        param_index += 1;
    }
    
    updates.push(format!("updated_at = ${}", param_index));
    param_index += 1;
    
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut string_idx = 0;
    
    if let Some(ref val) = identifier_type_id {
        params.push(val);
    }
    if value.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if plain_password.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if let Some(ref val) = is_enabled {
        params.push(val);
    }
    if comment.is_some() {
        params.push(&string_params[string_idx]);
        // No need to increment - this is the last string parameter
    }
    if let Some(ref val) = auth_attribute_group_id {
        params.push(val);
    }
    if let Some(ref val) = expiration_date {
        params.push(val);
    }
    if let Some(ref val) = reject_expired {
        params.push(val);
    }
    if let Some(ref val) = expired_auth_attribute_group_id {
        params.push(val);
    }
    params.push(&now);
    params.push(&id);
    
    let query = format!(
        "UPDATE user_identifiers SET {} WHERE id = ${}",
        updates.join(", "),
        param_index
    );
    
    let result = client.execute(&query, &params).await?;
    Ok(result > 0)
}

pub async fn delete_user_identifiers(pool: &Pool, user_id: i64) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        "DELETE FROM user_identifiers WHERE user_id = $1",
        &[&user_id],
    ).await?;
    Ok(())
}

pub async fn get_user_identifier(pool: &Pool, id: i64) -> Result<Option<Row>> {
    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id, user_id, identifier_type_id, value, is_enabled, comment, auth_attribute_group_id, expiration_date, reject_expired, expired_auth_attribute_group_id, created_at, updated_at 
         FROM user_identifiers 
         WHERE id = $1",
        &[&id],
    ).await?;
    
    Ok(row)
}

pub async fn get_user_identifiers(pool: &Pool, user_id: i64) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT id, identifier_type_id, value, is_enabled, comment, auth_attribute_group_id, expiration_date, reject_expired, expired_auth_attribute_group_id, created_at, updated_at 
         FROM user_identifiers 
         WHERE user_id = $1 
         ORDER BY created_at",
        &[&user_id],
    ).await?;
    
    Ok(rows)
}

// NAS Groups queries
pub async fn list_nas_groups(pool: &Pool) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT id, name, description, parent_id, created_at, updated_at 
         FROM nas_nas_group 
         ORDER BY name",
        &[],
    ).await?;
    
    Ok(rows)
}

pub async fn get_nas_group(pool: &Pool, id: i64) -> Result<Option<Row>> {
    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id, name, description, parent_id, created_at, updated_at 
         FROM nas_nas_group 
         WHERE id = $1",
        &[&id],
    ).await?;
    
    Ok(row)
}

pub async fn create_nas_group(
    pool: &Pool,
    name: &str,
    description: Option<&str>,
    parent_id: Option<i64>,
) -> Result<i64> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    let row = client.query_one(
        "INSERT INTO nas_nas_group (name, description, parent_id, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5) 
         RETURNING id",
        &[&name, &description, &parent_id, &now, &now],
    ).await?;
    
    Ok(row.get(0))
}

pub async fn update_nas_group(
    pool: &Pool,
    id: i64,
    name: Option<&str>,
    description: Option<&str>,
    parent_id: Option<i64>,
) -> Result<bool> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    // Two phase approach: collect values first, then build params
    let mut string_params: Vec<String> = Vec::new();
    let mut updates = Vec::new();
    let mut param_index = 1;
    
    if name.is_some() {
        updates.push(format!("name = ${}", param_index));
        string_params.push(name.unwrap().to_string());
        param_index += 1;
    }
    if description.is_some() {
        updates.push(format!("description = ${}", param_index));
        string_params.push(description.unwrap().to_string());
        param_index += 1;
    }
    if parent_id.is_some() {
        updates.push(format!("parent_id = ${}", param_index));
        param_index += 1;
    }
    
    updates.push(format!("updated_at = ${}", param_index));
    param_index += 1;
    
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
    let mut string_idx = 0;
    
    if name.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if description.is_some() {
        params.push(&string_params[string_idx]);
    }
    if let Some(ref val) = parent_id {
        params.push(val);
    }
    params.push(&now);
    params.push(&id);
    
    let query = format!(
        "UPDATE nas_nas_group SET {} WHERE id = ${}",
        updates.join(", "),
        param_index
    );
    
    let result = client.execute(&query, &params).await?;
    Ok(result > 0)
}

pub async fn delete_nas_group(pool: &Pool, id: i64) -> Result<bool> {
    let client = pool.get().await?;
    let result = client.execute("DELETE FROM nas_nas_group WHERE id = $1", &[&id]).await?;
    Ok(result > 0)
}

// NAS Devices queries
pub async fn list_nas_devices(pool: &Pool, page: i64, page_size: i64) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let offset = (page - 1) * page_size;
    
    let rows = client.query(
        "SELECT id, name, description, ip_address, coa_enabled, coa_port, vendor_id, secret_id, timezone_id, is_active, created_at, updated_at 
         FROM nas_nas 
         ORDER BY name 
         LIMIT $1 OFFSET $2",
        &[&page_size, &offset],
    ).await?;
    
    Ok(rows)
}

pub async fn count_nas_devices(pool: &Pool) -> Result<i64> {
    let client = pool.get().await?;
    let row = client.query_one("SELECT COUNT(*) FROM nas_nas", &[]).await?;
    Ok(row.get(0))
}

pub async fn get_nas_device(pool: &Pool, id: i64) -> Result<Option<Row>> {
    let client = pool.get().await?;
    let row = client.query_opt(
        "SELECT id, name, description, ip_address, coa_enabled, coa_port, vendor_id, secret_id, timezone_id, is_active, created_at, updated_at 
         FROM nas_nas 
         WHERE id = $1",
        &[&id],
    ).await?;
    
    Ok(row)
}

pub async fn create_nas_device(
    pool: &Pool,
    name: &str,
    description: Option<&str>,
    ip_address: &str,
    coa_enabled: bool,
    coa_port: i32,
    vendor_id: i64,
    secret_id: i64,
    timezone_id: i64,
    is_active: bool,
) -> Result<i64> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    let row = client.query_one(
        "INSERT INTO nas_nas (name, description, ip_address, coa_enabled, coa_port, vendor_id, secret_id, timezone_id, is_active, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) 
         RETURNING id",
        &[&name, &description, &ip_address, &coa_enabled, &coa_port, &vendor_id, &secret_id, &timezone_id, &is_active, &now, &now],
    ).await?;
    
    Ok(row.get(0))
}

pub async fn update_nas_device(
    pool: &Pool,
    id: i64,
    name: Option<&str>,
    description: Option<&str>,
    ip_address: Option<&str>,
    coa_enabled: Option<bool>,
    coa_port: Option<i32>,
    vendor_id: Option<i64>,
    secret_id: Option<i64>,
    timezone_id: Option<i64>,
    is_active: Option<bool>,
) -> Result<bool> {
    let client = pool.get().await?;
    let now = SystemTime::now();
    
    // Two phase approach: collect values first, then build params
    let mut string_params: Vec<String> = Vec::new();
    let mut updates = Vec::new();
    let mut param_index = 1;
    
    if name.is_some() {
        updates.push(format!("name = ${}", param_index));
        string_params.push(name.unwrap().to_string());
        param_index += 1;
    }
    if description.is_some() {
        updates.push(format!("description = ${}", param_index));
        string_params.push(description.unwrap().to_string());
        param_index += 1;
    }
    if ip_address.is_some() {
        updates.push(format!("ip_address = ${}", param_index));
        string_params.push(ip_address.unwrap().to_string());
        param_index += 1;
    }
    if coa_enabled.is_some() {
        updates.push(format!("coa_enabled = ${}", param_index));
        param_index += 1;
    }
    if coa_port.is_some() {
        updates.push(format!("coa_port = ${}", param_index));
        param_index += 1;
    }
    if vendor_id.is_some() {
        updates.push(format!("vendor_id = ${}", param_index));
        param_index += 1;
    }
    if secret_id.is_some() {
        updates.push(format!("secret_id = ${}", param_index));
        param_index += 1;
    }
    if timezone_id.is_some() {
        updates.push(format!("timezone_id = ${}", param_index));
        param_index += 1;
    }
    if is_active.is_some() {
        updates.push(format!("is_active = ${}", param_index));
        param_index += 1;
    }
    
    updates.push(format!("updated_at = ${}", param_index));
    param_index += 1;
    
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut string_idx = 0;
    
    if name.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if description.is_some() {
        params.push(&string_params[string_idx]);
        string_idx += 1;
    }
    if ip_address.is_some() {
        params.push(&string_params[string_idx]);
        // No need to increment - this is the last string parameter
    }
    if let Some(ref val) = coa_enabled {
        params.push(val);
    }
    if let Some(ref val) = coa_port {
        params.push(val);
    }
    if let Some(ref val) = vendor_id {
        params.push(val);
    }
    if let Some(ref val) = secret_id {
        params.push(val);
    }
    if let Some(ref val) = timezone_id {
        params.push(val);
    }
    if let Some(ref val) = is_active {
        params.push(val);
    }
    params.push(&now);
    params.push(&id);
    
    let query = format!(
        "UPDATE nas_nas SET {} WHERE id = ${}",
        updates.join(", "),
        param_index
    );
    
    let result = client.execute(&query, &params).await?;
    Ok(result > 0)
}

pub async fn delete_nas_device(pool: &Pool, id: i64) -> Result<bool> {
    let client = pool.get().await?;
    let result = client.execute("DELETE FROM nas_nas WHERE id = $1", &[&id]).await?;
    Ok(result > 0)
}

pub async fn set_nas_groups(pool: &Pool, nas_id: i64, group_ids: &[i64]) -> Result<()> {
    let client = pool.get().await?;
    
    // Delete existing associations
    client.execute(
        "DELETE FROM nas_nas_groups WHERE nas_id = $1",
        &[&nas_id],
    ).await?;
    
    // Insert new associations
    for group_id in group_ids {
        client.execute(
            "INSERT INTO nas_nas_groups (nas_id, nasgroup_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            &[&nas_id, group_id],
        ).await?;
    }
    
    Ok(())
}

pub async fn get_nas_groups(pool: &Pool, nas_id: i64) -> Result<Vec<i64>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT nasgroup_id FROM nas_nas_groups WHERE nas_id = $1",
        &[&nas_id],
    ).await?;
    
    Ok(rows.iter().map(|row| row.get(0)).collect())
}

// Vendors queries
pub async fn list_vendors(pool: &Pool) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT id, name, description, vendor_id, created_at, updated_at 
         FROM nas_vendor 
         ORDER BY name",
        &[],
    ).await?;
    
    Ok(rows)
}

// Secrets queries
pub async fn list_secrets(pool: &Pool) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT id, name, description, created_at, updated_at 
         FROM radius_secret 
         ORDER BY name",
        &[],
    ).await?;
    
    Ok(rows)
}

// User Identifier Types queries
pub async fn list_user_identifier_types(pool: &Pool) -> Result<Vec<Row>> {
    let client = pool.get().await?;
    let rows = client.query(
        "SELECT id, name, code, description, created_at, updated_at 
         FROM user_identifier_types 
         ORDER BY name",
        &[],
    ).await?;
    
    Ok(rows)
}

