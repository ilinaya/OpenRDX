# Stage 1: Build the application
FROM rust:1.86 as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the cargo manifest and lock to properly cache dependencies
COPY Cargo.toml Cargo.lock ./

# Fetch dependencies (useful for caching)
RUN cargo fetch

# Copy the source code
COPY . .

# Build the application in release mode
RUN cargo build --release

# Stage 2: Create minimal runtime container with the built app
FROM debian:bullseye-slim

# Install necessary dependencies (if any)
RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/MobileOffloadRadius /usr/app/

# Ensure the binary is executable
RUN chmod +x /usr/app/MobileOffloadRadius

# Expose RADIUS standard ports
EXPOSE 1812/udp
EXPOSE 1813/udp

# Define default command
CMD ["./MobileOffloadRadius"]