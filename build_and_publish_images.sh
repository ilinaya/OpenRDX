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

REGISTRY="ghcr.io"

# Avoid credential helper issues
export DOCKER_CONFIG=$(mktemp -d)

# Login to GHCR
echo "$PAT_TOKEN" | docker login "$REGISTRY" -u "$ORG" --password-stdin

# Check Docker environment
echo "Docker version: $(docker --version)"
echo "Checking buildx availability..."

# Try to determine if we can use buildx for multi-arch builds
USE_BUILDX=false
if command -v docker-buildx &>/dev/null; then
  echo "Found docker-buildx command"
  BUILDX_CMD="docker-buildx"
  USE_BUILDX=true
elif docker buildx version &>/dev/null; then
  echo "Found docker buildx capability"
  BUILDX_CMD="docker buildx"
  USE_BUILDX=true
else
  echo "⚠️ Buildx not available, will build for current architecture only"
fi

# Define image directories and names using simple arrays
IMAGE_DIRS=("core" "oss_backend" "oss_frontend" "radsec_proxy")
IMAGE_NAMES=("core" "oss_backend" "oss_frontend" "radsec_proxy")

# Build and push each
for i in "${!IMAGE_DIRS[@]}"; do
  local_dir="${IMAGE_DIRS[$i]}"
  image_name="${IMAGE_NAMES[$i]}"
  remote_image="${REGISTRY}/${ORG}/${NAMESPACE}/${image_name}"

  if [ "$USE_BUILDX" = true ]; then
    echo "🔨 Building multi-arch image and pushing $remote_image"
    
    # Try different buildx command patterns
    if $BUILDX_CMD build --platform linux/amd64,linux/arm64 -t "$remote_image:latest" --push "./$local_dir"; then
      echo "✅ Successfully built and pushed multi-arch $remote_image"
    else
      echo "⚠️ Buildx multi-arch build failed, falling back to standard build"
      USE_BUILDX=false
    fi
  fi
  
  # Fall back to standard build if buildx fails or isn't available
  if [ "$USE_BUILDX" = false ]; then
    echo "🔨 Building single-arch image $remote_image"
    if docker build -t "$remote_image:latest" "./$local_dir"; then
      echo "Pushing $remote_image:latest"
      if docker push "$remote_image:latest"; then
        echo "✅ Successfully pushed $remote_image"
      else
        echo "❌ Failed to push $remote_image"
        exit 1
      fi
    else
      echo "❌ Failed to build $remote_image"
      exit 1
    fi
  fi
done

echo "✅ All images built and pushed to $REGISTRY/$ORG/$NAMESPACE/"
if [ "$USE_BUILDX" = false ]; then
  echo "⚠️ Note: Images were built only for the current architecture ($(uname -m))"
else
  echo "✅ Images were built for multiple architectures (linux/amd64,linux/arm64)"
fi