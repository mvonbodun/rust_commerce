FROM lukemathwalker/cargo-chef:latest-rust-1.89 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y lld clang protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Plan dependencies using only manifests to maximize cache hits
FROM chef AS planner
# Copy manifests for path dependency and the service
COPY rust-common/Cargo.toml /app/rust-common/Cargo.toml
COPY shared-proto/Cargo.toml /app/shared-proto/Cargo.toml
COPY catalog/Cargo.toml /app/catalog/Cargo.toml
WORKDIR /app/catalog
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies and then the binary
FROM chef AS builder
# Work inside the service dir so relative paths in recipe.json (e.g., ../rust-common) resolve to /app/rust-common
WORKDIR /app/catalog
# Bring in the recipe computed in the previous stage
COPY --from=planner /app/catalog/recipe.json /app/catalog/recipe.json
# Provide path dependency sources before cooking deps
COPY rust-common/ /app/rust-common/
# Copy shared-proto for protobuf compilation
COPY shared-proto/ /app/shared-proto/
# Cook dependency layers only (this may create a temporary src with dummy main; we haven't copied our sources yet)
RUN cargo chef cook --release --recipe-path recipe.json
# Now copy full service sources
COPY catalog/ /app/catalog/
# Build the service binary
RUN cargo build --release --bin catalog-service

# Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# Copy the compiled binary
COPY --from=builder /app/catalog/target/release/catalog-service /app/catalog-service
# Sensible defaults (overridable via Compose)
ENV RUST_ENV=production
ENV RUST_LOG=info
# If the service exposes HTTP, uncomment the next line
# EXPOSE 8080
ENTRYPOINT ["/app/catalog-service"]
