FROM lukemathwalker/cargo-chef:latest-rust-1.89 AS chef
WORKDIR /app
RUN apt update && apt install lld clang protobuf-compiler -y

FROM chef AS planner
# Copy the rust-common library
COPY rust-common/ ./rust-common/
# Copy the catalog service
COPY catalog/ ./catalog/
# Set working directory to the service
WORKDIR /app/catalog
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json --bin catalog-service

FROM chef AS builder
# Copy the rust-common library first
COPY rust-common/ ./rust-common/
# Copy the catalog service
COPY catalog/ ./catalog/
# Set working directory to the service (needed for cargo chef cook)
WORKDIR /app/catalog
COPY --from=planner /app/catalog/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Build our project
RUN cargo build --release --bin catalog-service

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/catalog/target/release/catalog-service catalog-service
ENV RUST_ENV=production
ENTRYPOINT ["./catalog-service"]
