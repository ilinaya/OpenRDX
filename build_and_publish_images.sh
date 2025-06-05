#!/bin/bash
set -e

echo "🚀 Starting build process..."

# Load .env
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
else
  echo "❌ .env file not found."
  exit 1
fi

# Check required vars
if [[ -z "$PAT_TOKEN" || -z "$ORG" || -z "$NAMESPACE" ]]; then
  echo "❌ PAT_TOKEN, ORG, or NAMESPACE not set in .env"
  exit 1
fi

docker buildx version
docker buildx create --use

docker buildx build nginx/ \
  --platform linux/amd64,linux/arm64 \
  -t ghcr.io/ilinaya/openrdx/nginx:1.0.0 \
  -t ghcr.io/ilinaya/openrdx/nginx:latest \
  --push

docker buildx build oss_frontend/ \
  --platform linux/amd64,linux/arm64 \
  -t ghcr.io/ilinaya/openrdx/oss-frontend:1.0.0 \
  -t ghcr.io/ilinaya/openrdx/oss-frontend:latest \
  --push

docker buildx build oss_backend/ \
  --platform linux/amd64,linux/arm64 \
  -t ghcr.io/ilinaya/openrdx/oss-backend:latest \
  -t ghcr.io/ilinaya/openrdx/oss-backend:1.0.0 \
  --push

docker buildx build core/ \
  --platform linux/amd64,linux/arm64 \
  -t ghcr.io/ilinaya/openrdx/openrdx-core:1.0.0 \
  -t ghcr.io/ilinaya/openrdx/openrdx-core:latest \
  --push
