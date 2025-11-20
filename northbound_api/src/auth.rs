use actix_web::{dev::ServiceRequest, Error, HttpMessage, FromRequest, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use actix_web::web::Data;
use std::future::{ready, Ready, Future};
use std::pin::Pin;
use actix_web::dev::{forward_ready, Service, ServiceResponse, Transform};
use actix_web::http::header::AUTHORIZATION;
use log::{warn, debug};
use crate::error::ApiError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub api_key_id: Option<u64>,
    pub type_: String,
    pub created_by: Option<u64>,
    pub name: Option<String>,
    pub exp: i64,
    pub iat: i64,
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let claims = req.extensions().get::<Claims>().cloned();
        Box::pin(async move {
            claims.ok_or_else(|| {
                Error::from(ApiError::Authentication("Claims not found in request. Authentication required.".to_string()))
            })
        })
    }
}

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware { service }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Get JWT secret from app data
        let jwt_secret = match req.app_data::<Data<Arc<String>>>() {
            Some(secret) => secret.clone(),
            None => {
                warn!("JWT secret not found in app data");
                return Box::pin(async move {
                    Err(Error::from(ApiError::Internal("JWT secret not configured".to_string())))
                });
            }
        };

        // Extract token from Authorization header
        let auth_header = req.headers().get(AUTHORIZATION);
        
        if let Some(header_value) = auth_header {
            if let Ok(header_str) = header_value.to_str() {
                if header_str.starts_with("Bearer ") {
                    let token = &header_str[7..];
                    debug!("Verifying JWT token (length: {})", token.len());
                    
                    // Verify JWT token
                    let validation = Validation::new(Algorithm::HS256);
                    match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(jwt_secret.as_bytes()),
                        &validation,
                    ) {
                        Ok(token_data) => {
                            let claims = token_data.claims;
                            debug!("Token decoded successfully, API key ID: {:?}", claims.api_key_id);
                            
                            // Check if token is expired
                            let now = chrono::Utc::now().timestamp();
                            if claims.exp < now {
                                warn!("Token expired. exp={}, now={}", claims.exp, now);
                                return Box::pin(async move {
                                    Err(Error::from(ApiError::Authentication("Token expired".to_string())))
                                });
                            }

                            // Check if this is an API key token
                            if claims.type_ != "api_key" {
                                warn!("Invalid token type: {}", claims.type_);
                                return Box::pin(async move {
                                    Err(Error::from(ApiError::Authentication("Invalid token type. Expected 'api_key'".to_string())))
                                });
                            }

                            // Store claims in request extensions for use in handlers
                            // Use ReqData extractor which expects data in extensions
                            req.extensions_mut().insert(claims);
                        }
                        Err(e) => {
                            warn!("JWT decode error: {:?}", e);
                            return Box::pin(async move {
                                Err(Error::from(ApiError::Authentication(format!("Invalid token: {}", e))))
                            });
                        }
                    }
                } else {
                    return Box::pin(async move {
                        Err(Error::from(ApiError::Authentication("Invalid authorization header format. Expected 'Bearer <token>'".to_string())))
                    });
                }
            } else {
                return Box::pin(async move {
                    Err(Error::from(ApiError::Authentication("Invalid authorization header".to_string())))
                });
            }
        } else {
            return Box::pin(async move {
                Err(Error::from(ApiError::Authentication("Missing authorization header. Include 'Authorization: Bearer <token>'".to_string())))
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

