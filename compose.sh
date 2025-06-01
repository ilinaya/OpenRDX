#!/bin/bash
set -e

echo "🚀 Starting Docker Compose services..."

# Check if .env file exists
if [ -f .env ]; then
  echo "Loading environment variables from .env file..."
  export $(grep -v '^#' .env | xargs)
else
  echo "❌ No .env file found."
  exit 1
fi

# Verify API_URL is set
if [ -z "$API_URL" ]; then
  echo "❌ ERROR: API_URL environment variable is not set in .env file."
  echo "Please set API_URL in your .env file and try again."
  exit 1
fi

echo "✅ API_URL is set to: $API_URL"

# Determine docker compose command (supports both docker-compose and docker compose)
if command -v docker-compose &> /dev/null; then
  COMPOSE_CMD="docker-compose"
else
  COMPOSE_CMD="docker compose"
fi

# Get the current timestamp for build arguments
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Get the git commit SHA if available
if command -v git &> /dev/null && git rev-parse --is-inside-work-tree &> /dev/null; then
  COMMIT_SHA=$(git rev-parse --short HEAD)
else
  COMMIT_SHA="unknown"
fi

echo "🔨 Building services..."
echo "API_URL: $API_URL"
echo "Build timestamp: $TIMESTAMP"
echo "Commit SHA: $COMMIT_SHA"

# Create a temporary .env file for docker compose build
echo "API_URL=$API_URL" > .docker-env
echo "BUILD_TIMESTAMP=$TIMESTAMP" >> .docker-env
echo "COMMIT_SHA=$COMMIT_SHA" >> .docker-env

# this is just to pass Cargo Compilation, fake URL
echo "DATABASE_URL=postgres://placeholder/fake" >> .docker-env


# Build all services, ensuring .docker-env file is used
$COMPOSE_CMD --env-file .docker-env build

echo "🚀 Starting services in detached mode..."
$COMPOSE_CMD --env-file .docker-env up -d

# Clean up temporary env file
rm .docker-env

echo "✅ Services are now running in the background"
echo "📡 The oss_frontend service was built with API_URL: $API_URL"
echo "💡 To view logs: $COMPOSE_CMD logs -f"
echo "💡 To stop services: $COMPOSE_CMD down"

# Show running containers
echo "📋 Currently running containers:"
$COMPOSE_CMD ps