FROM lukemathwalker/cargo-chef:latest-rust-1.89 AS chef
WORKDIR /app
RUN apt update && apt install lld clang protobuf-compiler -y

FROM chef AS planner
# Copy the rust-common library
COPY rust-common/ ./rust-common/
# Copy the orders service
COPY orders/ ./orders/
# Set working directory to the service
WORKDIR /app/orders
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json --bin order-service

FROM chef AS builder
# Copy the rust-common library first
COPY rust-common/ ./rust-common/
# Copy the orders service
COPY orders/ ./orders/
# Set working directory to the service (needed for cargo chef cook)
WORKDIR /app/orders
COPY --from=planner /app/orders/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Build our project
RUN cargo build --release --bin order-service

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/orders/target/release/order-service order-service
ENV RUST_ENV=production
ENTRYPOINT ["./order-service"]
