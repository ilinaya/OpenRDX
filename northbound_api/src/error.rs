use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Authentication(msg) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "authentication_failed",
                    "message": msg
                }))
            }
            ApiError::NotFound(msg) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "not_found",
                    "message": msg
                }))
            }
            ApiError::Internal(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "internal_error",
                    "message": msg
                }))
            }
            ApiError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "bad_request",
                    "message": msg
                }))
            }
        }
    }
}

