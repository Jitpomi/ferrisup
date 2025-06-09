#!/bin/bash
set -e

# Start the Docker environment
echo "🚀 Starting Cloudflare Workers development environment..."
docker-compose up -d

echo "🔧 Running wrangler dev in container..."
docker-compose exec app bash -c "cd /app && wrangler dev --local"

# Keep the container running in the background
# Use ./stop.sh to stop the environment
