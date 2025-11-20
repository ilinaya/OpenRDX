use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Arc;
use log::info;
use config::Config;
use deadpool_postgres::Pool;

mod auth;
mod handlers;
mod models;
mod error;
mod openapi;
mod db;

use auth::JwtAuth;
use handlers::*;
use db::{Database, start_reconnect_task};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use openapi::ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let settings = Config::builder()
        .add_source(config::File::with_name("config/config").required(false))
        .add_source(config::Environment::with_prefix("NORTHBOUND"))
        .build()
        .expect("Failed to load configuration");

    let jwt_secret = std::env::var("API_KEY_JWT_SECRET")
        .or_else(|_| settings.get_string("jwt_secret"))
        .expect("API_KEY_JWT_SECRET or jwt_secret must be set");
    
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| {
            // Build from individual components
            let host = settings.get_string("db_host").unwrap_or_else(|_| "postgres".to_string());
            let port = settings.get_string("db_port").unwrap_or_else(|_| "5432".to_string());
            let user = settings.get_string("db_user").unwrap_or_else(|_| "postgres".to_string());
            let password = settings.get_string("db_password").unwrap_or_else(|_| "postgres".to_string());
            let dbname = settings.get_string("db_name").unwrap_or_else(|_| "postgres".to_string());
            Ok(format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, dbname))
        })
        .expect("DATABASE_URL or database components must be set");
    
    let bind_address = std::env::var("NORTHBOUND_BIND_ADDRESS")
        .or_else(|_| settings.get_string("bind_address"))
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    info!("Starting Northbound API on {}", bind_address);
    info!("JWT secret configured (length: {})", jwt_secret.len());
    info!("Connecting to database...");

    // Initialize database connection pool with exponential backoff
    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to database");
    
    let db_pool = Arc::new(database.pool);
    
    // Start reconnect task
    start_reconnect_task(db_pool.clone()).await;

    // Create shared state with JWT secret and database pool
    let jwt_secret = Arc::new(jwt_secret);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::from(jwt_secret.clone()))
            .app_data(web::Data::from(db_pool.clone()))
            .wrap(cors)
            .route("/health", web::get().to(health_check_public))
            .service(
                SwaggerUi::new("/swagger/{_:.*}")
                    .url("/api/v1/openapi.json", ApiDoc::openapi())
            )
            .service(
                web::scope("/api/v1")
                    .wrap(JwtAuth)
                    .route("/status", web::get().to(health_check))
                    // User endpoints
                    .route("/users", web::get().to(list_users))
                    .route("/users", web::post().to(create_user))
                    .route("/users/{id}", web::get().to(get_user))
                    .route("/users/{id}", web::put().to(update_user))
                    .route("/users/{id}", web::delete().to(delete_user))
                    // NAS Groups endpoints
                    .route("/nas-groups", web::get().to(list_nas_groups))
                    .route("/nas-groups", web::post().to(create_nas_group))
                    .route("/nas-groups/{id}", web::get().to(get_nas_group))
                    .route("/nas-groups/{id}", web::put().to(update_nas_group))
                    .route("/nas-groups/{id}", web::delete().to(delete_nas_group))
                    // NAS Devices endpoints
                    .route("/nas", web::get().to(list_nas))
                    .route("/nas", web::post().to(create_nas))
                    .route("/nas/{id}", web::get().to(get_nas))
                    .route("/nas/{id}", web::put().to(update_nas))
                    .route("/nas/{id}", web::delete().to(delete_nas))
                    // Vendor endpoints
                    .route("/vendors", web::get().to(list_vendors))
                    // Secret endpoints
                    .route("/secrets", web::get().to(list_secrets))
            )
    })
    .bind(bind_address)?
    .run()
    .await
}

