use actix_web::{web, HttpResponse, Result as ActixResult};
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use chrono::{DateTime, Utc};
use crate::auth::Claims;
use crate::models::*;
use crate::db::queries;
use log::{info, error};
use utoipa;

fn row_to_user(row: &Row) -> Result<User, Box<dyn std::error::Error>> {
    Ok(User {
        id: row.get::<_, i64>("id") as u64,
        email: row.get("email"),
        first_name: row.try_get("first_name").ok(),
        last_name: row.try_get("last_name").ok(),
        phone_number: row.try_get("phone_number").ok(),
        is_active: row.get("is_active"),
        external_id: row.try_get("external_id").ok(),
        allow_any_nas: row.try_get("allow_any_nas").ok(),
        group_ids: vec![],
        identifiers: vec![],
        created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
        updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
        last_login: row.try_get::<_, Option<DateTime<Utc>>>("last_login")
            .ok()
            .flatten()
            .map(|d| d.to_rfc3339()),
    })
}

fn row_to_nas(row: &Row) -> Result<Nas, Box<dyn std::error::Error>> {
    Ok(Nas {
        id: row.get::<_, i64>("id") as u64,
        name: row.get("name"),
        description: row.try_get("description").ok(),
        ip_address: row.get("ip_address"),
        coa_enabled: row.get("coa_enabled"),
        coa_port: row.get::<_, i32>("coa_port") as u16,
        vendor_id: row.try_get::<_, Option<i64>>("vendor_id").ok().flatten().map(|v| v as u64),
        secret_id: row.try_get::<_, Option<i64>>("secret_id").ok().flatten().map(|v| v as u64),
        timezone_id: row.try_get::<_, Option<i64>>("timezone_id").ok().flatten().map(|v| v as u64),
        group_ids: vec![],
        is_active: row.get("is_active"),
        created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
        updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
    })
}

#[utoipa::path(
    get,
    path = "/status",
    tag = "Health",
    security(
        ("Bearer" = [])
    ),
    responses(
        (status = 200, description = "API is healthy", body = HealthResponse),
        (status = 401, description = "Unauthorized - Invalid or missing token")
    )
)]
pub async fn health_check(claims: Claims) -> ActixResult<HttpResponse> {
    info!("Health check called by API key: {:?}", claims.api_key_id);
    
    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        api_key_id: claims.api_key_id,
        api_key_name: claims.name.clone(),
    }))
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy")
    )
)]
pub async fn health_check_public() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "northbound-api",
        "version": "1.0.0"
    })))
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    security(
        ("Bearer" = [])
    ),
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i64>, Query, description = "Page size (default: 10)")
    ),
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_users(
    claims: Claims,
    pool: web::Data<Pool>,
    query: web::Query<crate::models::PaginationQuery>,
) -> ActixResult<HttpResponse> {
    info!("List users called by API key: {:?}", claims.api_key_id);
    
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    
    match queries::list_users(pool.get_ref(), page, page_size).await {
        Ok(rows) => {
            let mut users = Vec::new();
            for row in rows {
                match row_to_user(&row) {
                    Ok(mut user) => {
                        // Get groups and identifiers
                        if let Ok(group_ids) = queries::get_user_groups(pool.get_ref(), user.id as i64).await {
                            user.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                        }
                        if let Ok(identifier_rows) = queries::get_user_identifiers(pool.get_ref(), user.id as i64).await {
                            user.identifiers = identifier_rows.iter()
                                .filter_map(|row| {
                                    Some(UserIdentifier {
                                        id: row.get::<_, i64>("id") as u64,
                                        identifier_type_id: row.get::<_, i64>("identifier_type_id") as u64,
                                        identifier_type_name: None,
                                        value: row.get("value"),
                                        is_enabled: row.get("is_enabled"),
                                        comment: row.try_get("comment").ok(),
                                        auth_attribute_group_id: row.try_get::<_, Option<i64>>("auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                        expiration_date: row.try_get::<_, Option<DateTime<Utc>>>("expiration_date").ok().flatten().map(|d| d.to_rfc3339()),
                                        reject_expired: row.get("reject_expired"),
                                        expired_auth_attribute_group_id: row.try_get::<_, Option<i64>>("expired_auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                        created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                                        updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                                    })
                                })
                                .collect();
                        }
                        users.push(user);
                    }
                    Err(e) => {
                        error!("Error converting row to user: {}", e);
                    }
                }
            }
            
            let count = queries::count_users(pool.get_ref()).await.unwrap_or(0) as u64;
            
            Ok(HttpResponse::Ok().json(UserListResponse {
                count,
                results: users,
            }))
        }
        Err(e) => {
            error!("Database error listing users: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to list users"))
        }
    }
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = User),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_user(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();
    info!("Get user {} called by API key: {:?}", user_id, claims.api_key_id);
    
    match queries::get_user(pool.get_ref(), user_id as i64).await {
        Ok(Some(row)) => {
            match row_to_user(&row) {
                Ok(mut user) => {
                    // Get groups and identifiers
                    if let Ok(group_ids) = queries::get_user_groups(pool.get_ref(), user_id as i64).await {
                        user.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                    }
                    if let Ok(identifier_rows) = queries::get_user_identifiers(pool.get_ref(), user_id as i64).await {
                        user.identifiers = identifier_rows.iter()
                            .filter_map(|row| {
                                Some(UserIdentifier {
                                    id: row.get::<_, i64>("id") as u64,
                                    identifier_type_id: row.get::<_, i64>("identifier_type_id") as u64,
                                    identifier_type_name: None,
                                    value: row.get("value"),
                                    is_enabled: row.get("is_enabled"),
                                    comment: row.try_get("comment").ok(),
                                    auth_attribute_group_id: row.try_get::<_, Option<i64>>("auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                    expiration_date: row.try_get::<_, Option<DateTime<Utc>>>("expiration_date").ok().flatten().map(|d| d.to_rfc3339()),
                                    reject_expired: row.get("reject_expired"),
                                    expired_auth_attribute_group_id: row.try_get::<_, Option<i64>>("expired_auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                                })
                            })
                            .collect();
                    }
                    Ok(HttpResponse::Ok().json(user))
                }
                Err(e) => {
                    error!("Error converting row to user: {}", e);
                    Err(actix_web::error::ErrorInternalServerError("Failed to process user"))
                }
            }
        }
        Ok(None) => Err(actix_web::error::ErrorNotFound("User not found")),
        Err(e) => {
            error!("Database error getting user: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to get user"))
        }
    }
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    security(
        ("Bearer" = [])
    ),
    request_body = UserCreate,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_user(
    claims: Claims,
    pool: web::Data<Pool>,
    payload: web::Json<UserCreate>,
) -> ActixResult<HttpResponse> {
    info!("Create user called by API key: {:?}", claims.api_key_id);
    
    let user_data = payload.into_inner();
    
    match queries::create_user(
        pool.get_ref(),
        &user_data.email,
        user_data.first_name.as_deref(),
        user_data.last_name.as_deref(),
        user_data.phone_number.as_deref(),
        user_data.external_id.as_deref(),
        user_data.is_active.unwrap_or(true),
        user_data.allow_any_nas,
    ).await {
        Ok(user_id) => {
            // Set groups
            if let Some(group_ids) = user_data.group_ids {
                if let Err(e) = queries::set_user_groups(pool.get_ref(), user_id, &group_ids.into_iter().map(|g| g as i64).collect::<Vec<_>>()).await {
                    error!("Failed to set user groups: {}", e);
                }
            }
            
            // Create identifiers
            if let Some(identifiers) = user_data.identifiers {
                for ident in identifiers {
                    let expiration_date = ident.expiration_date
                        .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                        .map(|d| d.with_timezone(&Utc));
                    
                    if let Err(e) = queries::create_user_identifier(
                        pool.get_ref(),
                        user_id,
                        ident.identifier_type_id as i64,
                        &ident.value,
                        ident.plain_password.as_deref(),
                        ident.is_enabled.unwrap_or(true),
                        ident.comment.as_deref(),
                        ident.auth_attribute_group_id.map(|v| v as i64),
                        expiration_date,
                        ident.reject_expired.unwrap_or(false),
                        ident.expired_auth_attribute_group_id.map(|v| v as i64),
                    ).await {
                        error!("Failed to create user identifier: {}", e);
                    }
                }
            }
            
            // Fetch the created user
            if let Ok(Some(row)) = queries::get_user(pool.get_ref(), user_id).await {
                if let Ok(mut user) = row_to_user(&row) {
                    if let Ok(group_ids) = queries::get_user_groups(pool.get_ref(), user_id).await {
                        user.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                    }
                    if let Ok(identifier_rows) = queries::get_user_identifiers(pool.get_ref(), user_id).await {
                        user.identifiers = identifier_rows.iter()
                            .filter_map(|row| {
                                Some(UserIdentifier {
                                    id: row.get::<_, i64>("id") as u64,
                                    identifier_type_id: row.get::<_, i64>("identifier_type_id") as u64,
                                    identifier_type_name: None,
                                    value: row.get("value"),
                                    is_enabled: row.get("is_enabled"),
                                    comment: row.try_get("comment").ok(),
                                    auth_attribute_group_id: row.try_get::<_, Option<i64>>("auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                    expiration_date: row.try_get::<_, Option<DateTime<Utc>>>("expiration_date").ok().flatten().map(|d| d.to_rfc3339()),
                                    reject_expired: row.get("reject_expired"),
                                    expired_auth_attribute_group_id: row.try_get::<_, Option<i64>>("expired_auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                                })
                            })
                            .collect();
                    }
                    return Ok(HttpResponse::Created().json(user));
                }
            }
            
            Err(actix_web::error::ErrorInternalServerError("Failed to retrieve created user"))
        }
        Err(e) => {
            error!("Database error creating user: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to create user"))
        }
    }
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    tag = "Users",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    request_body = UserUpdate,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_user(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
    payload: web::Json<UserUpdate>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();
    info!("Update user {} called by API key: {:?}", user_id, claims.api_key_id);
    
    let user_data = payload.into_inner();
    
    match queries::update_user(
        pool.get_ref(),
        user_id as i64,
        user_data.email.as_deref(),
        user_data.first_name.as_deref(),
        user_data.last_name.as_deref(),
        user_data.phone_number.as_deref(),
        user_data.external_id.as_deref(),
        user_data.is_active,
        user_data.allow_any_nas,
    ).await {
        Ok(true) => {
            // Update groups if provided
            if let Some(group_ids) = user_data.group_ids {
                if let Err(e) = queries::set_user_groups(pool.get_ref(), user_id as i64, &group_ids.into_iter().map(|g| g as i64).collect::<Vec<_>>()).await {
                    error!("Failed to update user groups: {}", e);
                }
            }
            
            // Update identifiers if provided (delete all and recreate)
            if let Some(identifiers) = user_data.identifiers {
                if let Err(e) = queries::delete_user_identifiers(pool.get_ref(), user_id as i64).await {
                    error!("Failed to delete existing identifiers: {}", e);
                }
                
                for ident in identifiers {
                    let expiration_date = ident.expiration_date
                        .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                        .map(|d| d.with_timezone(&Utc));
                    
                    if let Err(e) = queries::create_user_identifier(
                        pool.get_ref(),
                        user_id as i64,
                        ident.identifier_type_id as i64,
                        &ident.value,
                        ident.plain_password.as_deref(),
                        ident.is_enabled.unwrap_or(true),
                        ident.comment.as_deref(),
                        ident.auth_attribute_group_id.map(|v| v as i64),
                        expiration_date,
                        ident.reject_expired.unwrap_or(false),
                        ident.expired_auth_attribute_group_id.map(|v| v as i64),
                    ).await {
                        error!("Failed to create user identifier: {}", e);
                    }
                }
            }
            
            // Fetch the updated user
            if let Ok(Some(row)) = queries::get_user(pool.get_ref(), user_id as i64).await {
                if let Ok(mut user) = row_to_user(&row) {
                    if let Ok(group_ids) = queries::get_user_groups(pool.get_ref(), user_id as i64).await {
                        user.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                    }
                    if let Ok(identifier_rows) = queries::get_user_identifiers(pool.get_ref(), user_id as i64).await {
                        user.identifiers = identifier_rows.iter()
                            .filter_map(|row| {
                                Some(UserIdentifier {
                                    id: row.get::<_, i64>("id") as u64,
                                    identifier_type_id: row.get::<_, i64>("identifier_type_id") as u64,
                                    identifier_type_name: None,
                                    value: row.get("value"),
                                    is_enabled: row.get("is_enabled"),
                                    comment: row.try_get("comment").ok(),
                                    auth_attribute_group_id: row.try_get::<_, Option<i64>>("auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                    expiration_date: row.try_get::<_, Option<DateTime<Utc>>>("expiration_date").ok().flatten().map(|d| d.to_rfc3339()),
                                    reject_expired: row.get("reject_expired"),
                                    expired_auth_attribute_group_id: row.try_get::<_, Option<i64>>("expired_auth_attribute_group_id").ok().flatten().map(|v| v as u64),
                                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                                })
                            })
                            .collect();
                    }
                    return Ok(HttpResponse::Ok().json(user));
                }
            }
            Err(actix_web::error::ErrorInternalServerError("Failed to retrieve updated user"))
        }
        Ok(false) => Err(actix_web::error::ErrorNotFound("User not found")),
        Err(e) => {
            error!("Database error updating user: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to update user"))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "Users",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_user(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();
    info!("Delete user {} called by API key: {:?}", user_id, claims.api_key_id);
    
    match queries::delete_user(pool.get_ref(), user_id as i64).await {
        Ok(true) => Ok(HttpResponse::NoContent().finish()),
        Ok(false) => Err(actix_web::error::ErrorNotFound("User not found")),
        Err(e) => {
            error!("Database error deleting user: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to delete user"))
        }
    }
}

// NAS Groups handlers
#[utoipa::path(
    get,
    path = "/nas-groups",
    tag = "NAS Groups",
    security(
        ("Bearer" = [])
    ),
    responses(
        (status = 200, description = "List of NAS groups", body = Vec<NasGroup>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_nas_groups(
    claims: Claims,
    pool: web::Data<Pool>,
) -> ActixResult<HttpResponse> {
    info!("List NAS groups called by API key: {:?}", claims.api_key_id);
    
    match queries::list_nas_groups(pool.get_ref()).await {
        Ok(rows) => {
            let groups: Result<Vec<_>, _> = rows.iter().map(|row| {
                Ok(NasGroup {
                    id: row.get::<_, i64>("id") as u64,
                    name: row.get("name"),
                    description: row.try_get("description").ok(),
                    parent_id: row.try_get::<_, Option<i64>>("parent_id").ok().flatten().map(|v| v as u64),
                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                })
            }).collect();
            
            Ok(HttpResponse::Ok().json(groups.unwrap_or_default()))
        }
        Err(e) => {
            error!("Database error listing NAS groups: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to list NAS groups"))
        }
    }
}

#[utoipa::path(
    get,
    path = "/nas-groups/{id}",
    tag = "NAS Groups",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "NAS Group ID")
    ),
    responses(
        (status = 200, description = "NAS group details", body = NasGroup),
        (status = 404, description = "NAS group not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_nas_group(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
) -> ActixResult<HttpResponse> {
    let group_id = path.into_inner();
    
    match queries::get_nas_group(pool.get_ref(), group_id as i64).await {
        Ok(Some(row)) => {
            Ok(HttpResponse::Ok().json(NasGroup {
                id: row.get::<_, i64>("id") as u64,
                name: row.get("name"),
                description: row.try_get("description").ok(),
                parent_id: row.try_get::<_, Option<i64>>("parent_id").ok().flatten().map(|v| v as u64),
                created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
            }))
        }
        Ok(None) => Err(actix_web::error::ErrorNotFound("NAS group not found")),
        Err(e) => {
            error!("Database error getting NAS group: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to get NAS group"))
        }
    }
}

#[utoipa::path(
    post,
    path = "/nas-groups",
    tag = "NAS Groups",
    security(
        ("Bearer" = [])
    ),
    request_body = NasGroupCreate,
    responses(
        (status = 201, description = "NAS group created", body = NasGroup),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_nas_group(
    claims: Claims,
    pool: web::Data<Pool>,
    payload: web::Json<NasGroupCreate>,
) -> ActixResult<HttpResponse> {
    let group_data = payload.into_inner();
    
    match queries::create_nas_group(
        pool.get_ref(),
        &group_data.name,
        group_data.description.as_deref(),
        group_data.parent_id.map(|v| v as i64),
    ).await {
        Ok(group_id) => {
            if let Ok(Some(row)) = queries::get_nas_group(pool.get_ref(), group_id).await {
                return Ok(HttpResponse::Created().json(NasGroup {
                    id: row.get::<_, i64>("id") as u64,
                    name: row.get("name"),
                    description: row.try_get("description").ok(),
                    parent_id: row.try_get::<_, Option<i64>>("parent_id").ok().flatten().map(|v| v as u64),
                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                }));
            }
            Err(actix_web::error::ErrorInternalServerError("Failed to retrieve created NAS group"))
        }
        Err(e) => {
            error!("Database error creating NAS group: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to create NAS group"))
        }
    }
}

#[utoipa::path(
    put,
    path = "/nas-groups/{id}",
    tag = "NAS Groups",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "NAS Group ID")
    ),
    request_body = NasGroupUpdate,
    responses(
        (status = 200, description = "NAS group updated", body = NasGroup),
        (status = 404, description = "NAS group not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_nas_group(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
    payload: web::Json<NasGroupUpdate>,
) -> ActixResult<HttpResponse> {
    let group_id = path.into_inner();
    let group_data = payload.into_inner();
    
    match queries::update_nas_group(
        pool.get_ref(),
        group_id as i64,
        group_data.name.as_deref(),
        group_data.description.as_deref(),
        group_data.parent_id.map(|v| v as i64),
    ).await {
        Ok(true) => {
            if let Ok(Some(row)) = queries::get_nas_group(pool.get_ref(), group_id as i64).await {
                return Ok(HttpResponse::Ok().json(NasGroup {
                    id: row.get::<_, i64>("id") as u64,
                    name: row.get("name"),
                    description: row.try_get("description").ok(),
                    parent_id: row.try_get::<_, Option<i64>>("parent_id").ok().flatten().map(|v| v as u64),
                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                }));
            }
            Err(actix_web::error::ErrorInternalServerError("Failed to retrieve updated NAS group"))
        }
        Ok(false) => Err(actix_web::error::ErrorNotFound("NAS group not found")),
        Err(e) => {
            error!("Database error updating NAS group: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to update NAS group"))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/nas-groups/{id}",
    tag = "NAS Groups",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "NAS Group ID")
    ),
    responses(
        (status = 204, description = "NAS group deleted"),
        (status = 404, description = "NAS group not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_nas_group(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
) -> ActixResult<HttpResponse> {
    let group_id = path.into_inner();
    
    match queries::delete_nas_group(pool.get_ref(), group_id as i64).await {
        Ok(true) => Ok(HttpResponse::NoContent().finish()),
        Ok(false) => Err(actix_web::error::ErrorNotFound("NAS group not found")),
        Err(e) => {
            error!("Database error deleting NAS group: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to delete NAS group"))
        }
    }
}

// NAS Devices handlers
#[utoipa::path(
    get,
    path = "/nas",
    tag = "NAS",
    security(
        ("Bearer" = [])
    ),
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i64>, Query, description = "Page size (default: 10)")
    ),
    responses(
        (status = 200, description = "List of NAS devices", body = NasListResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_nas(
    claims: Claims,
    pool: web::Data<Pool>,
    query: web::Query<crate::models::PaginationQuery>,
) -> ActixResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);
    
    match queries::list_nas_devices(pool.get_ref(), page, page_size).await {
        Ok(rows) => {
            let mut nas_devices = Vec::new();
            for row in rows {
                match row_to_nas(&row) {
                    Ok(mut nas) => {
                        if let Ok(group_ids) = queries::get_nas_groups(pool.get_ref(), nas.id as i64).await {
                            nas.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                        }
                        nas_devices.push(nas);
                    }
                    Err(e) => {
                        error!("Error converting row to NAS: {}", e);
                    }
                }
            }
            
            let count = queries::count_nas_devices(pool.get_ref()).await.unwrap_or(0) as u64;
            
            Ok(HttpResponse::Ok().json(NasListResponse {
                count,
                results: nas_devices,
            }))
        }
        Err(e) => {
            error!("Database error listing NAS devices: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to list NAS devices"))
        }
    }
}

#[utoipa::path(
    get,
    path = "/nas/{id}",
    tag = "NAS",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "NAS Device ID")
    ),
    responses(
        (status = 200, description = "NAS device details", body = Nas),
        (status = 404, description = "NAS device not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_nas(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
) -> ActixResult<HttpResponse> {
    let nas_id = path.into_inner();
    
    match queries::get_nas_device(pool.get_ref(), nas_id as i64).await {
        Ok(Some(row)) => {
            match row_to_nas(&row) {
                Ok(mut nas) => {
                    if let Ok(group_ids) = queries::get_nas_groups(pool.get_ref(), nas_id as i64).await {
                        nas.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                    }
                    Ok(HttpResponse::Ok().json(nas))
                }
                Err(e) => {
                    error!("Error converting row to NAS: {}", e);
                    Err(actix_web::error::ErrorInternalServerError("Failed to process NAS device"))
                }
            }
        }
        Ok(None) => Err(actix_web::error::ErrorNotFound("NAS device not found")),
        Err(e) => {
            error!("Database error getting NAS device: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to get NAS device"))
        }
    }
}

#[utoipa::path(
    post,
    path = "/nas",
    tag = "NAS",
    security(
        ("Bearer" = [])
    ),
    request_body = NasCreate,
    responses(
        (status = 201, description = "NAS device created", body = Nas),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_nas(
    claims: Claims,
    pool: web::Data<Pool>,
    payload: web::Json<NasCreate>,
) -> ActixResult<HttpResponse> {
    let nas_data = payload.into_inner();
    
    match queries::create_nas_device(
        pool.get_ref(),
        &nas_data.name,
        nas_data.description.as_deref(),
        &nas_data.ip_address,
        nas_data.coa_enabled.unwrap_or(false),
        nas_data.coa_port.unwrap_or(3799) as i32,
        nas_data.vendor_id as i64,
        nas_data.secret_id as i64,
        nas_data.timezone_id as i64,
        nas_data.is_active.unwrap_or(true),
    ).await {
        Ok(nas_id) => {
            // Set groups
            if let Some(group_ids) = nas_data.group_ids {
                if let Err(e) = queries::set_nas_groups(pool.get_ref(), nas_id, &group_ids.into_iter().map(|g| g as i64).collect::<Vec<_>>()).await {
                    error!("Failed to set NAS groups: {}", e);
                }
            }
            
            // Fetch the created NAS device
            if let Ok(Some(row)) = queries::get_nas_device(pool.get_ref(), nas_id).await {
                match row_to_nas(&row) {
                    Ok(mut nas) => {
                        if let Ok(group_ids) = queries::get_nas_groups(pool.get_ref(), nas_id).await {
                            nas.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                        }
                        return Ok(HttpResponse::Created().json(nas));
                    }
                    Err(e) => {
                        error!("Error converting row to NAS: {}", e);
                    }
                }
            }
            Err(actix_web::error::ErrorInternalServerError("Failed to retrieve created NAS device"))
        }
        Err(e) => {
            error!("Database error creating NAS device: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to create NAS device"))
        }
    }
}

#[utoipa::path(
    put,
    path = "/nas/{id}",
    tag = "NAS",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "NAS Device ID")
    ),
    request_body = NasUpdate,
    responses(
        (status = 200, description = "NAS device updated", body = Nas),
        (status = 404, description = "NAS device not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_nas(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
    payload: web::Json<NasUpdate>,
) -> ActixResult<HttpResponse> {
    let nas_id = path.into_inner();
    let nas_data = payload.into_inner();
    
    match queries::update_nas_device(
        pool.get_ref(),
        nas_id as i64,
        nas_data.name.as_deref(),
        nas_data.description.as_deref(),
        nas_data.ip_address.as_deref(),
        nas_data.coa_enabled,
        nas_data.coa_port.map(|p| p as i32),
        nas_data.vendor_id.map(|v| v as i64),
        nas_data.secret_id.map(|v| v as i64),
        nas_data.timezone_id.map(|v| v as i64),
        nas_data.is_active,
    ).await {
        Ok(true) => {
            // Update groups if provided
            if let Some(group_ids) = nas_data.group_ids {
                if let Err(e) = queries::set_nas_groups(pool.get_ref(), nas_id as i64, &group_ids.into_iter().map(|g| g as i64).collect::<Vec<_>>()).await {
                    error!("Failed to update NAS groups: {}", e);
                }
            }
            
            // Fetch the updated NAS device
            if let Ok(Some(row)) = queries::get_nas_device(pool.get_ref(), nas_id as i64).await {
                match row_to_nas(&row) {
                    Ok(mut nas) => {
                        if let Ok(group_ids) = queries::get_nas_groups(pool.get_ref(), nas_id as i64).await {
                            nas.group_ids = group_ids.into_iter().map(|g| g as u64).collect();
                        }
                        return Ok(HttpResponse::Ok().json(nas));
                    }
                    Err(e) => {
                        error!("Error converting row to NAS: {}", e);
                    }
                }
            }
            Err(actix_web::error::ErrorInternalServerError("Failed to retrieve updated NAS device"))
        }
        Ok(false) => Err(actix_web::error::ErrorNotFound("NAS device not found")),
        Err(e) => {
            error!("Database error updating NAS device: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to update NAS device"))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/nas/{id}",
    tag = "NAS",
    security(
        ("Bearer" = [])
    ),
    params(
        ("id" = u64, Path, description = "NAS Device ID")
    ),
    responses(
        (status = 204, description = "NAS device deleted"),
        (status = 404, description = "NAS device not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_nas(
    claims: Claims,
    pool: web::Data<Pool>,
    path: web::Path<u64>,
) -> ActixResult<HttpResponse> {
    let nas_id = path.into_inner();
    
    match queries::delete_nas_device(pool.get_ref(), nas_id as i64).await {
        Ok(true) => Ok(HttpResponse::NoContent().finish()),
        Ok(false) => Err(actix_web::error::ErrorNotFound("NAS device not found")),
        Err(e) => {
            error!("Database error deleting NAS device: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to delete NAS device"))
        }
    }
}

// Vendors handlers
#[utoipa::path(
    get,
    path = "/vendors",
    tag = "Vendors",
    security(
        ("Bearer" = [])
    ),
    responses(
        (status = 200, description = "List of vendors", body = Vec<Vendor>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_vendors(
    claims: Claims,
    pool: web::Data<Pool>,
) -> ActixResult<HttpResponse> {
    match queries::list_vendors(pool.get_ref()).await {
        Ok(rows) => {
            let vendors: Result<Vec<_>, _> = rows.iter().map(|row| {
                Ok(Vendor {
                    id: row.get::<_, i64>("id") as u64,
                    name: row.get("name"),
                    description: row.try_get("description").ok(),
                    vendor_id: row.get::<_, i64>("vendor_id") as u64,
                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                })
            }).collect();
            
            Ok(HttpResponse::Ok().json(vendors.unwrap_or_default()))
        }
        Err(e) => {
            error!("Database error listing vendors: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to list vendors"))
        }
    }
}

// Secrets handlers
#[utoipa::path(
    get,
    path = "/secrets",
    tag = "Secrets",
    security(
        ("Bearer" = [])
    ),
    responses(
        (status = 200, description = "List of secrets", body = Vec<Secret>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_secrets(
    claims: Claims,
    pool: web::Data<Pool>,
) -> ActixResult<HttpResponse> {
    match queries::list_secrets(pool.get_ref()).await {
        Ok(rows) => {
            let secrets: Result<Vec<_>, _> = rows.iter().map(|row| {
                Ok(Secret {
                    id: row.get::<_, i64>("id") as u64,
                    name: row.get("name"),
                    description: row.try_get("description").ok(),
                    created_at: row.get::<_, DateTime<Utc>>("created_at").to_rfc3339(),
                    updated_at: row.get::<_, DateTime<Utc>>("updated_at").to_rfc3339(),
                })
            }).collect();
            
            Ok(HttpResponse::Ok().json(secrets.unwrap_or_default()))
        }
        Err(e) => {
            error!("Database error listing secrets: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to list secrets"))
        }
    }
}
