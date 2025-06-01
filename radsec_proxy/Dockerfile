# Build stage
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/OpenRDX-Radsec-Proxy .

COPY --from=builder /usr/src/app/docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Create a non-root user
RUN useradd -m -u 1000 radius

# Switch to non-root user
USER radius

# Set environment variables
ENV RUST_LOG=info

EXPOSE 2083

ENTRYPOINT ["docker-entrypoint.sh"]