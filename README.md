![OpenRDX Logo](logo_openrdx.png)

OpenRDX is a modern, scalable platform for managing and processing RADIUS authentication and accounting data. Built with a microservices architecture, it provides a robust solution for network access control and accounting.

## Technology Stack

### Frontend
- **Angular 19+**: Modern, component-based UI framework
- **Material Design**: For consistent and responsive UI components
- **TypeScript**: For type-safe development
- **RxJS**: For reactive programming and state management

### Backend
- **Django 5+**: High-level Python web framework
- **Django REST Framework**: For building RESTful APIs
- **Celery**: For asynchronous task processing
- **Redis**: For caching and message broker
- **MongoDB**: For RADIUS accounting data storage

### Core Service
- **Rust**: High-performance RADIUS server implementation
- **Tokio**: Asynchronous runtime
- **SQLx**: Type-safe SQL query builder
- **MongoDB**: For accounting data storage

### Data Storage
- **PostgreSQL**: Primary relational database for user data, configurations, and system state
- **MongoDB**: Specialized storage for RADIUS accounting data and logs
- **Redis**: Caching layer and message broker

## Project Structure

```
.
├── core/               # Rust-based RADIUS service
├── oss_backend/        # Django REST Framework API
├── oss_frontend/       # Angular frontend application
├── k8s/               # Kubernetes configurations
├── nginx/             # Nginx reverse proxy configuration
│   ├── conf.d/        # Nginx server configurations
│   └── ssl/           # SSL certificates
├── scripts/           # Utility scripts
├── docker-compose.yml # Docker Compose configuration
└── README.md
```

## Prerequisites

- Docker and Docker Compose
- Kubernetes cluster (for k8s deployment)
- Rust toolchain (for local development)
- Python 3.12+ (for local development)
- Node.js 22+ (for local development)
- mkcert (for local SSL certificates)
- MongoDB 6.0+ (for accounting data)

## Quick Start with Docker Compose

1. Clone the repository:
   ```bash
   git clone https://github.com/ilinaya/OpenRDX.git
   cd OpenRDX
   ```

2. Copy the example environment file and configure it:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration, API_URL mainly for the FE <> BE communication
   ```

3. Generate SSL certificates for BE / FE SSL:
   ```bash
   ./scripts/generate-ssl.sh
   ```

4. Start all services using the compose script:
   ```bash
   ./compose.sh
   ```

5. Access the services:
   - Frontend: http://<API_URL>
   - Backend API: http://<API_URL>/api
   - Core Service (RADIUS): UDP ports 1812, 1813
   - Core Service (RadSEC): TCP port 2083
   - PostgreSQL: Internal only (port 5432)
   - MongoDB: Internal only (port 27017)
   - Redis: Internal only (port 6379)

6. Default Authentication Credentials (FE):
   ```json
   {
     "email": "admin@example.com",
     "password": "admin"
   }
   ```
   ⚠️ **Important**: These are default development credentials. You must change them immediately after first login for security reasons.

### Resetting Admin Password

If you need to reset an admin user's password, you can use the Django management command:

**Via Docker Compose:**
```bash
docker-compose exec oss_backend python manage.py reset_admin_password <email> <new_password>
```

**Via Docker:**
```bash
docker exec -it openrdx-backend python manage.py reset_admin_password <email> <new_password>
```

**Example:**
```bash
docker-compose exec oss_backend python manage.py reset_admin_password admin@example.com newSecurePassword123
```

This command will:
- Find the admin user by email address
- Validate the new password according to Django's password validation rules
- Hash the password using Django's secure password hashing (pbkdf2_sha256)
- Update the user's password in the database

**Note**: The password must meet Django's password validation requirements (minimum 8 characters, not too common, etc.).

### Password Change via UI

Logged-in users can change their own password through the web interface:
1. Click on your email address in the top navigation bar
2. Select "Change Password" from the dropdown menu
3. Enter your current password and new password
4. Confirm the new password and submit

## Architecture

### Components
- **Core Services**:
  - **Authentication Service (Rust)**:
    - High-performance RADIUS authentication server
    - Handles PAP, CHAP, MS-CHAP, MS-CHAPv2 authentication
    - Exposes UDP port 1812
    - Horizontally scalable under load balancer
    - Environment variables:
      ```env
      # Service type
      SERVICE_TYPE=auth
      
      # RADIUS settings
      RADIUS_BIND_ADDR=0.0.0.0:1812
      
      # Database settings
      DATABASE_URL=postgresql://user:pass@postgres:5432/radius
      REDIS_URL=redis://redis:6379/0
      ```

  - **Accounting Service (Rust)**:
    - High-performance RADIUS accounting server
    - Handles Start/Stop/Interim-Update accounting records
    - Exposes UDP port 1813
    - Horizontally scalable under load balancer
    - Stores accounting data in MongoDB
    - Environment variables:
      ```env
      # Service type
      SERVICE_TYPE=acct
      
      # RADIUS settings
      RADIUS_BIND_ADDR=0.0.0.0:1813
      
      # Database settings
      MONGODB_URI=mongodb://mongodb:27017/radius_accounting
      REDIS_URL=redis://redis:6379/0
      ```

  - **RadSec Proxy Service (Rust) (Optional)**:
    - Secure RADIUS over TLS implementation
    - Handles both authentication and accounting over TLS
    - Exposes TCP port 2083
    - Horizontally scalable under load balancer
    - Supports TLS 1.2/1.3
    - Environment variables:
      ```env
      # RadSec settings
      RADSEC_BIND_ADDR=0.0.0.0:2083
      
      # Database settings
      DATABASE_URL=postgresql://user:pass@postgres:5432/radius
      MONGODB_URI=mongodb://mongodb:27017/radius_accounting
      REDIS_URL=redis://redis:6379/0
      ```

  - **Load Balancer Configuration (Optional)**:
    - UDP load balancing for RADIUS (1812/1813)
    - TCP load balancing for RadSec (2083)
    - Session persistence for accounting
    - Health checks for service availability
    - Automatic failover
    - Example Kubernetes Service:
      ```yaml
      apiVersion: v1
      kind: Service
      metadata:
        name: radius-auth
      spec:
        type: LoadBalancer
        ports:
        - protocol: UDP
          port: 1812
          targetPort: 1812
        selector:
          app: radius-auth
      ```
      
- **Backend Service (Django)**:
  - RESTful API built with Django REST Framework
  - User management and system configuration
  - Accessible through Nginx at /api
  - Internal service, not directly accessible from outside
  - Uses PostgreSQL for relational data
  - Integrates with MongoDB for accounting data
  - Health check endpoint available at `/health` for Kubernetes and Docker health monitoring
  - Environment variables:
    ```env
    # Django settings
    DEBUG=False
    SECRET_KEY=django-insecure-key-for-development-only
    ALLOWED_HOSTS=localhost,127.0.0.1,*
    LOGGING_LEVEL=INFO
    DJANGO_SETTINGS_MODULE=core.settings

    # Database settings
    DB_NAME=postgres
    DB_USER=postgres
    DB_PASSWORD=postgres
    DB_HOST=postgres
    DB_PORT=5432

    # MongoDB settings
    MONGODB_URI=mongodb://mongodb:27017/radius_accounting

    # JWT settings
    JWT_SECRET_KEY=jwt-secret-key-for-development-only
    JWT_ACCESS_TOKEN_LIFETIME=60  # minutes
    JWT_REFRESH_TOKEN_LIFETIME=1  # days

    # Email settings (Mailgun)
    MAILGUN_API_KEY=
    MAILGUN_SENDER_DOMAIN=mg.example.com
    DEFAULT_FROM_EMAIL=noreply@example.com

    # Integrations
    GOOGLE_CHAT_WEBHOOK_URL=

    # Redis settings
    REDIS_URL=redis://redis:6379/0

    # RADIUS settings
    COA_TOPIC=radius_coa  # Topic used for Change of Authorization (CoA) requests from backend to core
    ```

- **Frontend Service (Angular)**:
  - Modern single-page application
  - Material Design components
  - Real-time updates using WebSocket
  - Served through Nginx at root path
  - Internal service, not directly accessible from outside

- **Nginx Reverse Proxy**:
  - Handles all HTTP/HTTPS traffic
  - Manages SSL/TLS termination
  - Routes traffic to appropriate services
  - Handles CORS headers
  - Provides security headers

- **PostgreSQL**:
  - Primary relational database
  - Stores user data, configurations, and system state
  - Internal service only
  - Persistent storage
  - Environment variables:
    ```env
    POSTGRES_USER=postgres
    POSTGRES_PASSWORD=postgres
    POSTGRES_DB=openrdx
    ```

- **MongoDB**:
  - Specialized storage for RADIUS accounting
  - High-performance logging and analytics
  - Internal service only
  - Persistent storage
  - Environment variables:
    ```env
    MONGO_INITDB_DATABASE=radius_accounting
    ```

- **Redis**:
  - Caching layer
  - Message broker for Celery
  - Session storage
  - Internal service only
  - Persistent storage

### Security Features
- SSL/TLS encryption for all HTTP traffic
- Automatic HTTP to HTTPS redirection
- CORS headers for API access
- Internal services not exposed to public
- Secure SSL configuration (TLSv1.2, TLSv1.3)
- Proper file permissions for SSL certificates
- JWT-based authentication
- Rate limiting and DDoS protection

## Kubernetes Deployment

1. Apply the Kubernetes configurations:
   ```bash
   kubectl apply -f k8s/
   ```

2. Add the following to your `/etc/hosts`:
   ```
   127.0.0.1 openrdx.local
   ```

3. Access the services through the ingress:
   - Frontend: https://openrdx.local
   - Backend API: https://openrdx.local/api

## SSL Certificate Management

### Local Development
The project handles SSL certificates automatically through the nginx container:

1. **Automatic Self-Signed Certificates**:
   - If no SSL certificates are found in `nginx/ssl/`, the nginx container will automatically generate self-signed certificates during startup
   - Generated certificates will be placed in `nginx/ssl/`
   - This is suitable for local development and testing

2. **Custom SSL Certificates**:
   - For production or custom certificates, place your SSL files in `nginx/ssl/`:
     ```
     nginx/ssl/
     ├── server.crt    # Your SSL certificate
     └── server.key    # Your private key
     ```
   - Update the SSL configuration in `nginx/conf.d/` to match your certificate filenames:
     ```nginx
     ssl_certificate     /etc/nginx/ssl/server.crt;
     ssl_certificate_key /etc/nginx/ssl/server.key;
     ```

3. **Certificate Requirements**:
   - Certificates must be in PEM format
   - Private key must be unencrypted
   - Files must have proper permissions (readable by nginx)

### Production Deployment
For production environments:
1. Use proper SSL certificates from a trusted Certificate Authority
2. Place certificates in `nginx/ssl/` before deployment
3. Ensure proper security measures for certificate storage and rotation
4. Consider using a certificate management solution (e.g., Let's Encrypt, AWS Certificate Manager)

## Development

### Core Service (Rust)
```bash
cd core
cargo build
cargo run
```

### Backend Service (Django)
```bash
cd oss_backend
python -m venv venv
source venv/bin/activate  # or `venv\Scripts\activate` on Windows
pip install -r requirements.txt
python manage.py runserver
```

### Frontend Service (Angular)
```bash
cd oss_frontend
npm install
ng serve
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to submit pull requests, report issues, and contribute to the project.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Security

Please report any security issues to security@openrdx.org

## Support

For support, please open an issue in the GitHub repository or contact support@openrdx.org

# OpenRDX Core

A comprehensive Authentication, Authorization, and Accounting (AAA) system with support for multiple authentication protocols.

## Features

- **Multi-protocol Authentication Support**
  - PAP (Password Authentication Protocol)
  - CHAP (Challenge Handshake Authentication Protocol)
  - MS-CHAP (Microsoft Challenge Handshake Authentication Protocol)
  - MS-CHAPv2 (Microsoft Challenge Handshake Authentication Protocol Version 2)

- **RADIUS Server**
  - Standards-compliant RADIUS implementation
  - Support for vendor-specific attributes
  - Secure password handling
  - Message integrity verification
  - Account status checking

- **User Management**
  - User creation and management
  - Password management
  - Account status control
  - Identifier management

- **Security**
  - Secure password storage
  - Challenge-response authentication
  - Message authentication
  - Vendor-specific attribute handling

## Documentation

- [Getting Started](docs/getting-started.md)
- [Features](docs/features.md)
- [Components](docs/components.md)
- [Services](docs/services.md)
- [RADIUS Server](docs/radius-server.md)
- [MS-CHAPv2 Missing Features](docs/mschapv2-missing-features.md) - Implementation guide for future MS-CHAPv2 enhancements
- [Contributing](docs/contributing.md)

## Quick Start

1. Clone the repository
2. Set up the environment variables
3. Run the database migrations
4. Start the services

```bash
# Set up environment
cp .env.example .env
# Edit .env with your configuration

# Run migrations
cargo run --bin migrate

# Start the services
cargo run --bin radius-server
```

## Configuration

### Required Environment Variables

```env
# Database
DATABASE_URL=postgres://user:password@localhost/dbname

# RADIUS Server
RADIUS_BIND_ADDR=0.0.0.0:1812
RADIUS_SECRET=your_shared_secret
```

## Development

### Prerequisites

- Rust 1.70 or later
- PostgreSQL 13 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Please read [CONTRIBUTING.md](docs/contributing.md) for details on our code of conduct and the process for submitting pull requests.
