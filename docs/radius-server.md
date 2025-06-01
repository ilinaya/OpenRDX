# RADIUS Server

The OpenRDX RADIUS server is implemented in Rust and provides high-performance authentication and accounting services.

## Features

- Authentication methods:
  - PAP (Password Authentication Protocol)
  - CHAP (Challenge Handshake Authentication Protocol)
  - MS-CHAP (Microsoft Challenge Handshake Authentication Protocol)
  - MS-CHAPv2 (Microsoft Challenge Handshake Authentication Protocol v2)
  - EAP (Extensible Authentication Protocol)
    - EAP-TLS
    - EAP-TTLS
    - EAP-PEAP
    - EAP-SIM
    - EAP-AKA
    - EAP-AKA'

- Accounting support
  - Start/Stop records
  - Interim updates
  - Session tracking
  - MongoDB storage

- RadSec (RADIUS over TLS) support via RadSec Proxy
  - Secure RADIUS communication
  - TLS 1.2/1.3 support
  - Certificate-based authentication
  - Client and server modes

## Configuration

### Core Service

The RADIUS server is configured through environment variables:

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

# Logging
LOG_LEVEL=info
```

### RadSec Proxy

The RadSec proxy is configured through environment variables:

```env
# RadSec settings
RADSEC_LISTEN_PORT=2083
RADSEC_TLS_CERT=/etc/radsecproxy/certs/server.crt
RADSEC_TLS_KEY=/etc/radsecproxy/certs/server.key
RADSEC_TLS_CA=/etc/radsecproxy/certs/ca.crt
```

## Usage

### Standard RADIUS

For standard RADIUS communication, clients should connect to UDP ports 1812 (authentication) and 1813 (accounting).

### RadSec

For secure RADIUS communication over TLS:

1. Configure your RADIUS client to use RadSec:
   ```
   radsec {
     server radsec.example.com:2083 {
       secret your_shared_secret
       tls {
         ca_file /path/to/ca.crt
         cert_file /path/to/client.crt
         key_file /path/to/client.key
       }
     }
   }
   ```

2. The RadSec proxy will handle the TLS connection and forward requests to the core service.

## Security Considerations

- Always use strong shared secrets
- Keep certificates and private keys secure
- Use TLS 1.2 or higher for RadSec
- Regularly rotate certificates
- Monitor for suspicious activity
- Use proper firewall rules to restrict access

## Monitoring

The RADIUS server provides detailed logging for:
- Authentication attempts
- Accounting records
- TLS handshakes
- Error conditions

Logs are stored in MongoDB for analysis and auditing.

## Troubleshooting

Common issues and solutions:

1. Authentication failures:
   - Check shared secrets
   - Verify user credentials
   - Check certificate validity for EAP methods

2. RadSec connection issues:
   - Verify TLS certificates
   - Check firewall rules
   - Ensure proper TLS version support

3. Accounting problems:
   - Check MongoDB connection
   - Verify accounting port access
   - Check for duplicate session IDs 