FROM rust:1.75-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Set environment variables (will be overridden by docker-compose)
ENV BASIC_MEM0_API_KEY=default_key_please_override
ENV BIND_ADDRESS=0.0.0.0:8000

# Build the specific binary
RUN cargo build --bin open-context-vault

# Expose port
EXPOSE 8000

# Run the binary directly from target directory
CMD ["./target/debug/open-context-vault"]
