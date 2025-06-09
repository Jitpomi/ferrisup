#!/bin/bash
set -e

# Start the Docker environment if not already running
if [ -z "$(docker-compose ps -q app 2>/dev/null)" ]; then
  echo "🚀 Starting Fastly Compute@Edge build environment..."
  docker-compose up -d
fi

# Build the application in the container
echo "🔨 Building Fastly Compute@Edge application..."
docker-compose exec app bash -c "cd /app && fastly compute build"

echo "✅ Build completed successfully!"
