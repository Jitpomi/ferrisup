#!/bin/bash

# Stop the Docker environment
echo "🛑 Stopping Fastly Compute@Edge development environment..."
docker-compose down

echo "✅ Environment stopped successfully!"
