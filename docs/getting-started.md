# Getting Started

This guide will help you set up and run OpenRDX on your local machine or in a containerized environment.

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
   git clone https://github.com/ilinaya/OpenRDX.git
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

5. Default Authentication Credentials:
   ```json
   {
     "email": "admin@example.com",
     "password": "admin"
   }
   ```
   ⚠️ **Important**: These are default development credentials. Change them after first login.

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

## Local Development

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

## SSL Certificate Management

See the README or scripts/generate-ssl.sh for details on local and production SSL setup.

## Project Structure

```
oss_frontend/
├── src/
│   ├── app/
│   │   ├── core/           # Core module (services, guards, interceptors)
│   │   ├── features/       # Feature modules
│   │   ├── shared/         # Shared components and utilities
│   │   └── app.module.ts   # Root module
│   ├── assets/            # Static assets
│   └── environments/      # Environment configurations
├── docs/                  # Documentation
└── tests/                # Test files
```

## Common Issues and Solutions

### Node Version Issues

If you encounter node version issues, use nvm to switch to the correct version:
```bash
nvm use 18
```

### Port Already in Use

If port 4200 is already in use, you can specify a different port:
```bash
npm start -- --port 4201
```

### Build Errors

If you encounter build errors:
1. Clear the cache: `npm cache clean --force`
2. Delete node_modules: `rm -rf node_modules`
3. Reinstall dependencies: `npm install`

## Next Steps

- Read the [Features](./features.md) documentation to learn about available functionality
- Check the [Components](./components.md) guide for detailed component documentation
- Review the [API Integration](./api-integration.md) guide for backend integration details 