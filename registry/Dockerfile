# Multi-stage build using cargo-chef for dependency caching.
# TL;DR: separates dependency builds from app builds so we don't rebuild Rust deps on every code change.

FROM rust:latest as chef
WORKDIR /app
RUN cargo install cargo-chef

# Generate a dependency recipe. This is basically a lockfile that cargo-chef uses to build
# deps in isolation. The whole point is so Docker caches the dep layer separately.
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build deps here. This layer stays cached as long as Cargo.toml doesn't change.
FROM chef as builder
# cmake is required by aws-lc-rs. Install it before copying recipe so this layer caches independently.
# (Moving this below COPY would bust the cache every time we touch code.)
RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
# This is the slow part. Once this finishes, it's cached until Cargo.toml changes.
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy the actual code and build the app. This layer rebuilds on every code change,
# but the deps are already baked in from above.
COPY . .
RUN cargo build --release --bin registry

# Strip it down for runtime. No build tools, no Rust compiler, just the binary.
FROM debian:testing-slim AS runtime
WORKDIR /app
# Need CA certs for HTTPS (R2, registry calls, etc). libssl-dev for runtime TLS.
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/registry /usr/local/bin
ENV PORT=3000
EXPOSE 3000
CMD ["/usr/local/bin/registry"]