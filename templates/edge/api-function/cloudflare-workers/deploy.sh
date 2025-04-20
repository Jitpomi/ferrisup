#!/bin/bash
set -e

# Start the Docker environment if not already running
if [ -z "$(docker-compose ps -q app 2>/dev/null)" ]; then
  echo "🚀 Starting Cloudflare Workers deployment environment..."
  docker-compose up -d
fi

# Check if user is logged in to Cloudflare
docker-compose exec app bash -c "if ! wrangler whoami &>/dev/null; then echo '⚠️ Please login to Cloudflare first using: docker-compose exec app wrangler login'; exit 1; fi"

# Deploy the application
echo "🚀 Deploying Cloudflare Workers application..."
docker-compose exec app bash -c "cd /app && wrangler deploy"

echo "✅ Deployment completed successfully!"
