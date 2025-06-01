#!/bin/sh
set -e

CERT_DIR=/etc/nginx/ssl
CERT_FILE=$CERT_DIR/cert.pem
KEY_FILE=$CERT_DIR/key.pem

mkdir -p $CERT_DIR

if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
  echo "🔐 SSL certificates not found — generating self-signed cert..."
  openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout "$KEY_FILE" -out "$CERT_FILE" \
    -subj "/C=US/ST=NA/L=NA/O=OpenRDX/CN=localhost"
  echo "✅ Self-signed certs generated at $CERT_DIR"
else
  echo "🔒 Existing SSL certificates found, using them"
fi

# Start nginx
exec nginx -g "daemon off;"