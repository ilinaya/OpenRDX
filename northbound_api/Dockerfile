### Builder stage (Nightly for latest features)
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev build-essential \
 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY src ./src
COPY config ./config

# Generate Cargo.lock if it doesn't exist, then build
RUN cargo generate-lockfile || true
RUN cargo build --release && strip target/release/northbound-api || true

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

