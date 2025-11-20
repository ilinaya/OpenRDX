# Northbound API

The Northbound API is a high-performance REST API built with Actix-web (Rust) that provides programmatic access to the OpenRDX RADIUS management system. It uses JWT-based authentication via API keys generated from the main backend.

## Features

- **JWT Authentication**: Verifies tokens signed by the same secret as the backend API key generation
- **PostgreSQL Integration**: Direct database access using deadpool-postgres connection pooling
- **Persistent Connections**: Connection pool with exponential backoff and automatic reconnection
- **High Performance**: Built with Rust and Actix-web for maximum performance
- **OpenAPI/Swagger Documentation**: Interactive API documentation via Swagger UI using utoipa
- **CORS Support**: Cross-origin requests are supported
- **Health Check Endpoints**: Public and authenticated health check endpoints
- **Full CRUD Operations**: Complete Create, Read, Update, Delete operations for:
  - Users (with identities and user groups)
  - User Groups (full CRUD)
  - User Identifiers (update)
  - User Identifier Types (list)
  - NAS Groups (full CRUD)
  - NAS Devices (with secrets, vendors, groups, timezones)
  - Vendors (list)
  - Secrets (list)

## Architecture

The Northbound API provides direct database access for external applications to interact with OpenRDX. It:

1. Accepts requests with JWT Bearer tokens
2. Verifies tokens using the same `API_KEY_JWT_SECRET` as the backend
3. Extracts claims from the JWT (API key ID, creator, etc.)
4. Connects directly to PostgreSQL database using connection pooling
5. Performs CRUD operations on database tables
6. Returns formatted JSON responses

The API uses deadpool-postgres for connection pooling with:
- Exponential backoff on connection failures
- Automatic reconnection monitoring
- Persistent connection pool
- Health checks every 30 seconds

## Getting Started

### Prerequisites

- Rust (nightly) for Rust Edition 2024
- PostgreSQL database (shared with backend)
- Docker and Docker Compose (for containerized deployment)

### Configuration

The Northbound API requires the following environment variables:

- `API_KEY_JWT_SECRET`: The JWT secret used to verify API key tokens (must match backend)
- `DATABASE_URL`: PostgreSQL connection string (e.g., `postgres://user:password@host:5432/dbname`)
  - Or build from components: `db_host`, `db_port`, `db_user`, `db_password`, `db_name`
- `NORTHBOUND_BIND_ADDRESS`: Address to bind to (default: `0.0.0.0:8080`)

### Building

```bash
cd northbound_api
cargo build --release
```

### Running

```bash
export API_KEY_JWT_SECRET=your_secret_here
cargo run
```

### Docker

The API is included in `docker-compose.yml` and will automatically use the `API_KEY_JWT_SECRET` environment variable from the compose file.

## API Endpoints

### Public Endpoints

- `GET /health` - Public health check (no authentication required)
- `GET /swagger` - Swagger UI documentation
- `GET /api/v1/openapi.json` - OpenAPI specification

### Authenticated Endpoints (Require Bearer Token)

All endpoints under `/api/v1` require authentication via JWT Bearer token:

#### Users
- `GET /api/v1/users` - List all users (with pagination: `?page=1&page_size=10`)
- `POST /api/v1/users` - Create a new user (with identities and groups)
- `GET /api/v1/users/{id}` - Get user by ID
- `PUT /api/v1/users/{id}` - Update user (with identities and groups)
- `DELETE /api/v1/users/{id}` - Delete user

#### User Groups
- `GET /api/v1/user-groups` - List all user groups
- `POST /api/v1/user-groups` - Create a new user group
- `GET /api/v1/user-groups/{id}` - Get user group by ID
- `PUT /api/v1/user-groups/{id}` - Update user group
- `DELETE /api/v1/user-groups/{id}` - Delete user group

#### User Identifiers
- `PUT /api/v1/user-identifiers/{id}` - Update user identifier

#### User Identifier Types
- `GET /api/v1/user-identifier-types` - List all user identifier types

#### NAS Groups
- `GET /api/v1/nas-groups` - List all NAS groups
- `POST /api/v1/nas-groups` - Create a new NAS group
- `GET /api/v1/nas-groups/{id}` - Get NAS group by ID
- `PUT /api/v1/nas-groups/{id}` - Update NAS group
- `DELETE /api/v1/nas-groups/{id}` - Delete NAS group

#### NAS Devices
- `GET /api/v1/nas` - List all NAS devices (with pagination: `?page=1&page_size=10`)
- `POST /api/v1/nas` - Create a new NAS device (with secret, vendor, groups, timezone)
- `GET /api/v1/nas/{id}` - Get NAS device by ID
- `PUT /api/v1/nas/{id}` - Update NAS device (with secret, vendor, groups, timezone)
- `DELETE /api/v1/nas/{id}` - Delete NAS device

#### Vendors
- `GET /api/v1/vendors` - List all vendors

#### Secrets
- `GET /api/v1/secrets` - List all secrets

## Authentication

All authenticated endpoints require a JWT Bearer token in the Authorization header:

```
Authorization: Bearer <your_api_key_jwt_token>
```

The token must:
- Be signed with the same `API_KEY_JWT_SECRET` as configured in the backend
- Have `type: "api_key"` in the payload
- Not be expired (checked via `exp` claim)

## API Documentation

### Accessing Swagger UI

Once the service is running, access the Swagger UI at:
- Local: `http://localhost:8080/swagger`
- Via Nginx: `https://your-domain/northbound-api/swagger`

### Using the API

1. Generate an API key from the OpenRDX web interface (Settings → API Keys)
2. Copy the generated JWT token
3. Include it in requests: `Authorization: Bearer <token>`
4. Use the Swagger UI to test endpoints interactively

## Development

### Project Structure

```
northbound_api/
├── src/
│   ├── main.rs          # Application entry point and routing
│   ├── auth.rs          # JWT authentication middleware
│   ├── handlers.rs      # Request handlers (CRUD operations)
│   ├── models.rs        # Data models with utoipa schemas
│   ├── error.rs         # Error types
│   ├── openapi.rs       # OpenAPI specification using utoipa
│   └── db/
│       ├── mod.rs       # Database connection pool with backoff
│       └── queries.rs   # Database queries
├── config/
│   └── config.toml      # Configuration file
├── Cargo.toml           # Rust dependencies
├── Dockerfile           # Docker build configuration
└── README.md            # This file
```

### Adding New Endpoints

1. Add the handler function in `src/handlers.rs`
2. Register the route in `src/main.rs`
3. Add OpenAPI documentation in `src/openapi.rs`
4. Update this README

## Docker Compose Integration

The Northbound API is automatically configured in `docker-compose.yml`:

- Service name: `northbound_api`
- Internal port: `8080`
- Accessible via Nginx at: `/northbound-api/`
- Uses the same `API_KEY_JWT_SECRET` as the backend
- Connects to the same PostgreSQL database as the backend
- Depends on `postgres` and `oss_backend` services

## Security Considerations

- JWT tokens are verified on every request
- Expired tokens are automatically rejected
- Only tokens with `type: "api_key"` are accepted
- CORS is configured to allow cross-origin requests (configure for production)

## Performance

The Northbound API is built for high performance:
- Asynchronous request handling with Tokio
- Connection pooling with deadpool-postgres
- Exponential backoff on connection failures
- Automatic reconnection monitoring
- Efficient JWT verification
- Minimal memory allocations
- Built with Rust for safety and speed
- Direct database access (no backend API overhead)

## Troubleshooting

### JWT Verification Fails

- Ensure `API_KEY_JWT_SECRET` matches the backend configuration
- Check that the token is not expired
- Verify the token format: `Bearer <token>`

### Connection Issues

- Check that the service is running: `docker-compose ps`
- Verify network connectivity: `docker-compose exec northbound_api curl localhost:8080/health`
- Check logs: `docker-compose logs northbound_api`

## License

MIT License

