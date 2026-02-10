# Build Stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Install build dependencies (cmake needed for aws-lc-rs)
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake && rm -rf /var/lib/apt/lists/*

# Pre-build dependencies to cache them
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Now copy the real source and build
COPY . .
RUN cargo build --release

# Runtime Stage
FROM debian:testing-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/registry .

ENV PORT=3000
EXPOSE 3000

CMD ["./registry"]