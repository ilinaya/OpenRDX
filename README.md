# OpenRDX

OpenRDX is a modern, scalable platform for managing and processing RADIUS authentication and accounting data. Built with a microservices architecture, it provides a robust solution for network access control and accounting.

## Technology Stack

### Frontend
- **Angular 16+**: Modern, component-based UI framework
- **Material Design**: For consistent and responsive UI components
- **TypeScript**: For type-safe development
- **RxJS**: For reactive programming and state management

### Backend
- **Django 4.2+**: High-level Python web framework
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
- Python 3.8+ (for local development)
- Node.js 16+ (for local development)
- mkcert (for local SSL certificates)
- MongoDB 6.0+ (for accounting data)

## Quick Start with Docker Compose

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/OpenRDX.git
   cd OpenRDX
   ```

2. Generate SSL certificates for local development:
   ```bash
   ./scripts/generate-ssl.sh
   ```

3. Start all services:
   ```bash
   docker-compose up -d
   ```

4. Access the services:
   - Frontend: https://localhost
   - Backend API: https://localhost/api
   - Core Service (RADIUS): UDP ports 1812, 1813
   - PostgreSQL: Internal only (port 5432)
   - MongoDB: Internal only (port 27017)
   - Redis: Internal only (port 6379)

5. Default Authentication Credentials:
   ```json
   {
     "email": "admin@example.com",
     "password": "admin"
   }
   ```
   ⚠️ **Important**: These are default development credentials. You must change them immediately after first login for security reasons.

## Architecture

### Components
- **Core Service (Rust)**:
  - High-performance RADIUS server
  - Handles authentication and accounting
  - Exposes UDP ports 1812, 1813
  - Internal service, not directly accessible from outside
  - Stores accounting data in MongoDB
  - Uses MongoDB for high-performance logging
  - Environment variables:
    ```env
    # RADIUS settings
    COA_TOPIC=radius_coa  # Topic used for Change of Authorization (CoA) requests
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
The project includes automatic SSL certificate generation for local development:
```bash
./scripts/generate-ssl.sh
```
This script:
- Installs mkcert if not present
- Generates self-signed certificates
- Places them in the correct location
- Sets proper permissions

### Production
For production environments, you have two options:

1. Using Let's Encrypt with certbot:
   ```bash
   # Install certbot
   sudo apt-get update
   sudo apt-get install certbot

   # Generate certificates
   sudo certbot certonly --standalone -d yourdomain.com

   # Copy certificates to nginx/ssl/
   sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem nginx/ssl/cert.pem
   sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem nginx/ssl/key.pem
   ```

2. Using Docker with certbot:
   ```bash
   docker run -it --rm \
     -v nginx/ssl:/etc/nginx/ssl \
     -p 80:80 -p 443:443 \
     certbot/certbot certonly --standalone -d yourdomain.com
   ```

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
