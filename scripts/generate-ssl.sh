#!/bin/bash

# Create SSL directory if it doesn't exist
mkdir -p nginx/ssl

# Check if mkcert is installed
if ! command -v mkcert &> /dev/null; then
    echo "mkcert is not installed. Installing..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install mkcert
        brew install nss  # for Firefox
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo apt install libnss3-tools
        sudo apt install mkcert
    else
        echo "Unsupported OS. Please install mkcert manually."
        exit 1
    fi
fi

# Install local CA
mkcert -install

# Generate certificates for local development
mkcert -cert-file nginx/ssl/cert.pem -key-file nginx/ssl/key.pem localhost 127.0.0.1 ::1

# Set proper permissions
chmod 644 nginx/ssl/cert.pem
chmod 600 nginx/ssl/key.pem

echo "SSL certificates generated successfully!"
echo "Certificates are located in nginx/ssl/" 