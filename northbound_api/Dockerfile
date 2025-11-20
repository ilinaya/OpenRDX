### Builder stage (Nightly for latest features)
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev build-essential \
 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY src ./src
COPY config ./config

# Generate Cargo.lock if it doesn't exist
RUN cargo generate-lockfile || true

# Build the project (this will fail if build fails)
RUN cargo build --release || (echo "Build failed! Showing errors:" && cargo build --release 2>&1 | head -50 && exit 1)

# Cargo converts hyphens to underscores in binary names
# So "northbound-api" becomes "northbound_api"
RUN ls -la target/release/ && \
    if [ -f target/release/northbound_api ]; then \
        echo "Found binary: northbound_api (hyphen converted to underscore)"; \
        strip target/release/northbound_api; \
        cp target/release/northbound_api target/release/northbound-api; \
    elif [ -f target/release/northbound-api ]; then \
        echo "Found binary: northbound-api"; \
        strip target/release/northbound-api; \
    else \
        echo "ERROR: Binary not found! Available files:"; \
        ls -la target/release/; \
        exit 1; \
    fi

### Runtime stage (matches GLIBC from builder)
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/northbound-api /usr/local/bin/northbound-api
COPY config/config.toml /config/config.toml

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/northbound-api"]

CMD []

