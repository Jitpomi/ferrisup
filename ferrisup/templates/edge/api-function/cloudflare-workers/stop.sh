#!/bin/bash

# Stop the Docker environment
echo "🛑 Stopping Cloudflare Workers development environment..."
docker-compose down

echo "✅ Environment stopped successfully!"
