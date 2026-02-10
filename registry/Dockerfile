# 1. Base Stage: Cargo Chef
# We use cargo-chef to calculate the build plan for dependencies
FROM rust:latest as chef
WORKDIR /app
RUN cargo install cargo-chef

# 2. Planner Stage
# Compute a lock-file recipe
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 3. Builder Stage (Cached Dependencies)
# Build dependencies first - this layer is cached until Cargo.toml changes
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Install cmake (needed for aws-lc-rs)
RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*
# Build dependencies - this is the slow part, but now it's cached!
RUN cargo chef cook --release --recipe-path recipe.json

# Build the main app
COPY . .
RUN cargo build --release --bin registry

# 4. Runtime Stage
# Minimal runtime image
FROM debian:testing-slim AS runtime
WORKDIR /app
# Install OpenSSL/CA certs
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/registry /usr/local/bin
ENV PORT=3000
EXPOSE 3000
CMD ["/usr/local/bin/registry"]