FROM lukemathwalker/cargo-chef:latest-rust-1.89 AS chef
WORKDIR /app
RUN apt update && apt install lld clang protobuf-compiler -y

FROM chef AS planner
# Copy the rust-common library
COPY rust-common/ ./rust-common/
# Copy the inventory service
COPY inventory/ ./inventory/
# Set working directory to the service
WORKDIR /app/inventory
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json --bin inventory-service

FROM chef AS builder
# Copy the rust-common library first
COPY rust-common/ ./rust-common/
# Copy the inventory service
COPY inventory/ ./inventory/
# Set working directory to the service (needed for cargo chef cook)
WORKDIR /app/inventory
COPY --from=planner /app/inventory/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Build our project
RUN cargo build --release --bin inventory-service

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/inventory/target/release/inventory-service inventory-service
ENV RUST_ENV=production
ENTRYPOINT ["./inventory-service"]
