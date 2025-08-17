FROM lukemathwalker/cargo-chef:latest-rust-1.89 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y lld clang protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Plan dependencies using only manifests to maximize cache hits
FROM chef AS planner
# Copy manifests for path dependency and the service
COPY rust-common/Cargo.toml /app/rust-common/Cargo.toml
COPY orders/Cargo.toml /app/orders/Cargo.toml
WORKDIR /app/orders
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies and then the binary
FROM chef AS builder
# Work inside the service dir so relative paths in recipe.json (e.g., ../rust-common) resolve to /app/rust-common
WORKDIR /app/orders
# Bring in the recipe computed in the previous stage
COPY --from=planner /app/orders/recipe.json /app/orders/recipe.json
# Provide path dependency sources before cooking deps
COPY rust-common/ /app/rust-common/
# Cook dependency layers only (no app sources yet)
RUN cargo chef cook --release --recipe-path recipe.json
# Now copy full service sources
COPY orders/ /app/orders/
# Build the service binary
RUN cargo build --release --bin order-service

# Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Copy the compiled binary
COPY --from=builder /app/orders/target/release/order-service /app/order-service
# Sensible defaults (overridable via Compose)
ENV RUST_ENV=production
ENV RUST_LOG=info
ENTRYPOINT ["/app/order-service"]
