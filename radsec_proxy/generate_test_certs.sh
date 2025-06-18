#!/bin/bash

# Generate private key
openssl genrsa -out server.key 2048

# Generate CSR
openssl req -new -key server.key -out server.csr -subj "/CN=localhost"

# Generate self-signed certificate
openssl x509 -req -days 365 -in server.csr -signkey server.key -out server.crt

# Convert to base64
CERT_BASE64=$(base64 -i server.crt)
KEY_BASE64=$(base64 -i server.key)

# Create .env file
cat > .env << EOL
RADSEC_BIND_ADDR=0.0.0.0:2083
RADSEC_CERT_BASE64=${CERT_BASE64}
RADSEC_KEY_BASE64=${KEY_BASE64}
RADIUS_SERVER=radius.example.com:1812
RADIUS_SECRET=testing123
EOL

# Clean up temporary files
rm server.key server.csr server.crt

echo "Generated .env file with test certificates" 