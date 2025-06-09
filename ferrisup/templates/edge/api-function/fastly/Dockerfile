FROM rust:1.69.0-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    ca-certificates \
    npm \
    nodejs \
    && rm -rf /var/lib/apt/lists/*

# Add the wasm32-wasi target
RUN rustup target add wasm32-wasi

# Install Fastly CLI using npm (more reliable than direct binary download)
RUN npm install -g @fastly/cli

# Create app directory
WORKDIR /app

# Copy project files
COPY . .

# Build the project
RUN cargo build --target wasm32-wasi

# Default command to keep container running
CMD ["bash"]
