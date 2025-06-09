#!/bin/bash
set -e

# Start the Docker environment if not already running
if [ -z "$(docker-compose ps -q app 2>/dev/null)" ]; then
  echo "🚀 Starting Fastly Compute@Edge deployment environment..."
  docker-compose up -d
fi

# Check if user is logged in to Fastly
docker-compose exec app bash -c "if ! fastly whoami &>/dev/null; then echo '⚠️ Please login to Fastly first using: docker-compose exec app fastly profile create'; exit 1; fi"

# Deploy the application
echo "🚀 Deploying Fastly Compute@Edge application..."
docker-compose exec app bash -c "cd /app && fastly compute publish"

echo "✅ Deployment completed successfully!"
