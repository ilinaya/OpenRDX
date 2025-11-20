use utoipa::OpenApi;
use crate::models::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "OpenRDX Northbound API",
        version = "1.0.0",
        description = "REST API for programmatic access to OpenRDX RADIUS management system. This API allows external applications to interact with OpenRDX using JWT-based authentication. All endpoints require authentication via Bearer token in the Authorization header.",
        contact(
            name = "ILINAIA Team",
            email = "support@ilinaia.com"
        ),
        license(name = "MIT")
    ),
    servers(
        (url = "/northbound-api/api/v1", description = "Production API v1 (via Nginx)"),
        (url = "/api/v1", description = "Development API v1 (direct)")
    ),
    paths(
        crate::handlers::health_check,
        crate::handlers::health_check_public,
        crate::handlers::list_users,
        crate::handlers::get_user,
        crate::handlers::create_user,
        crate::handlers::update_user,
        crate::handlers::delete_user,
        crate::handlers::list_nas_groups,
        crate::handlers::get_nas_group,
        crate::handlers::create_nas_group,
        crate::handlers::update_nas_group,
        crate::handlers::delete_nas_group,
        crate::handlers::list_nas,
        crate::handlers::get_nas,
        crate::handlers::create_nas,
        crate::handlers::update_nas,
        crate::handlers::delete_nas,
        crate::handlers::list_vendors,
        crate::handlers::list_secrets
    ),
    components(schemas(
        HealthResponse,
        User,
        UserIdentifier,
        UserCreate,
        UserIdentifierCreate,
        UserUpdate,
        UserListResponse,
        UserGroup,
        NasGroup,
        NasGroupCreate,
        NasGroupUpdate,
        Nas,
        NasCreate,
        NasUpdate,
        NasListResponse,
        Vendor,
        Secret,
        ErrorResponse,
        PaginationQuery
    )),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Users", description = "User management endpoints"),
        (name = "NAS Groups", description = "NAS group management endpoints"),
        (name = "NAS", description = "NAS device management endpoints"),
        (name = "Vendors", description = "Vendor listing endpoints"),
        (name = "Secrets", description = "Secret listing endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modifier for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "Bearer",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT token obtained from the API key management interface"))
                        .build(),
                ),
            )
        }
    }
}
