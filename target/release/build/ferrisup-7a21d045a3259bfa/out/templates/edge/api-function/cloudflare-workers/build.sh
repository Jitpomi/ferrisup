#!/bin/bash
set -e

# Start the Docker environment if not already running
if [ -z "$(docker-compose ps -q app 2>/dev/null)" ]; then
  echo "🚀 Starting Cloudflare Workers build environment..."
  docker-compose up -d
fi

# Build the application in the container
echo "🔨 Building Cloudflare Workers application..."
docker-compose exec app bash -c "cd /app && cargo build --release"

echo "✅ Build completed successfully!"
