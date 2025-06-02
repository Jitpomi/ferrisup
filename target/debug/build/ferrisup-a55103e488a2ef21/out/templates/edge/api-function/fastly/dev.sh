#!/bin/bash
set -e

# Start the Docker environment
echo "🚀 Starting Fastly Compute@Edge development environment..."
docker-compose up -d

echo "🔧 Running Fastly Compute@Edge local server..."
docker-compose exec app bash -c "cd /app && fastly compute serve --listen-addr=0.0.0.0:7676"

# Keep the container running in the background
# Use ./stop.sh to stop the environment
