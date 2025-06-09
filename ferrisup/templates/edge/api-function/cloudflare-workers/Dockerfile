FROM rust:1.69.0-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install wrangler CLI
RUN npm install -g wrangler

# Create app directory
WORKDIR /app

# Copy project files
COPY . .

# Set up the project
RUN cargo build

# Default command to keep container running
CMD ["bash"]
