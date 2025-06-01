# Getting Started

This guide will help you get started with OpenRDX, from installation to running your first authentication.

## Prerequisites

- Docker and Docker Compose
- Git
- mkcert (for local SSL certificates)
- Basic understanding of RADIUS protocol

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/ilinaya/OpenRDX.git
   cd OpenRDX
   ```

2. Create and configure environment variables:
   ```bash
   cp .env.example .env
   # Edit .env file with your configuration
   ```

3. Generate SSL certificates:
   ```bash
   ./scripts/generate-ssl.sh
   ```

## Running the Services

### Using the Compose Script

The easiest way to start all services is using the provided `compose.sh` script:

```bash
./compose.sh
```

This script will:
1. Check for required environment variables
2. Build all services with proper build arguments
3. Start services in detached mode
4. Show running containers

The script requires:
- A valid `.env` file
- The `API_URL` environment variable to be set
- Docker or Docker Compose installed

### Manual Start

Alternatively, you can start services manually:

```bash
docker-compose up -d
```

## Verifying the Installation

1. Check service status:
   ```bash
   docker-compose ps
   ```

2. Access the web interface:
   - Frontend: https://localhost
   - Backend API: https://localhost/api

3. Test RADIUS authentication:
   ```bash
   # Using radtest (install with: apt-get install freeradius-utils)
   radtest user password localhost 0 testing123
   ```

4. Test RadSec connection:
   ```bash
   # Using radtest with TLS
   radtest -t tls user password localhost:2083 0 testing123
   ```

## Configuration

### Core Service

The RADIUS server is configured through environment variables in `.env`:

```env
# RADIUS settings
RADIUS_AUTH_PORT=1812
RADIUS_ACCT_PORT=1813
RADIUS_SECRET=your_shared_secret

# Database settings
DB_HOST=postgres
DB_PORT=5432
DB_NAME=openrdx
DB_USER=postgres
DB_PASSWORD=postgres

# MongoDB settings
MONGODB_URI=mongodb://mongodb:27017/radius_accounting
```

### RadSec Proxy

Configure RadSec settings in `.env`:

```env
# RadSec settings
RADSEC_LISTEN_PORT=2083
RADSEC_TLS_CERT=/etc/radsecproxy/certs/server.crt
RADSEC_TLS_KEY=/etc/radsecproxy/certs/server.key
RADSEC_TLS_CA=/etc/radsecproxy/certs/ca.crt
```

## Next Steps

1. Create your first user:
   - Log in to the web interface
   - Navigate to Users section
   - Add a new user with appropriate permissions

2. Configure your first NAS:
   - Add NAS details in the web interface
   - Configure shared secret
   - Set allowed authentication methods

3. Test authentication:
   - Use radtest or your preferred RADIUS client
   - Verify accounting records
   - Check logs for any issues

## Troubleshooting

Common issues and solutions:

1. Services not starting:
   - Check Docker logs: `docker-compose logs`
   - Verify environment variables
   - Check port availability

2. Authentication failures:
   - Verify user credentials
   - Check shared secrets
   - Review server logs

3. RadSec connection issues:
   - Verify TLS certificates
   - Check firewall rules
   - Ensure proper TLS version support

For more detailed troubleshooting, see the [Troubleshooting Guide](troubleshooting.md). 