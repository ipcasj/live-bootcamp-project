# Start with image that has the Rust toolchain installed
FROM rust:1.85-alpine AS chef
USER root
# Add cargo-chef and all build dependencies (openssl-dev, pkgconf, musl-dev)
RUN apk add --no-cache musl-dev pkgconf openssl-dev protobuf && cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Capture info needed to build dependencies
RUN apk add --no-cache openssl-dev pkgconf musl-dev protobuf
RUN cargo chef prepare --recipe-path recipe.json
COPY . .
# Capture info needed to build dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apk add --no-cache openssl-dev pkgconf musl-dev protobuf
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin auth-service

# We do not need the Rust toolchain to run the binary!
# Start with a minimal image and copy over the binary and assets folder.
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/auth-service /usr/local/bin
COPY --from=builder /app/assets /app/assets
ENTRYPOINT ["/usr/local/bin/auth-service"]