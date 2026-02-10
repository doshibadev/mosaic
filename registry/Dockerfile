# Build Stage
FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL is needed for HTTPS registry requests)
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/registry .

# Set environment variables
ENV PORT=3000
EXPOSE 3000

# Run the binary
CMD ["./registry"]
