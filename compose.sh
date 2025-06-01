
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

# Backup the original docker-compose.yml file if not already backed up
if [ ! -f docker-compose.yml.bak ]; then
  echo "Creating backup of docker-compose.yml..."
  cp docker-compose.yml docker-compose.yml.bak
fi

# Update docker-compose.yml to use the API_URL from environment
echo "Updating docker-compose.yml to use API_URL from environment variable..."
sed -i.tmp -E "s|([ ]*- API_URL=).*|\1\${API_URL}|g" docker-compose.yml
rm -f docker-compose.yml.tmp

# Determine docker compose command (supports both docker-compose and docker compose)
if command -v docker-compose &> /dev/null; then
  COMPOSE_CMD="docker-compose"
else
  COMPOSE_CMD="docker compose"
fi

echo "🔨 Building services with API_URL=$API_URL..."
$COMPOSE_CMD build

echo "🚀 Starting services in detached mode..."
$COMPOSE_CMD up -d

echo "✅ Services are now running in the background"
echo "📡 The oss_frontend service is using API_URL: $API_URL"
echo "💡 To view logs: $COMPOSE_CMD logs -f"
echo "💡 To stop services: $COMPOSE_CMD down"

# Show running containers
echo "📋 Currently running containers:"
$COMPOSE_CMD ps