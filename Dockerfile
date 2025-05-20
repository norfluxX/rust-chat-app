# Build stage
FROM rust:1.75-slim AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/rust-chat-app /usr/local/bin/rust-chat-app

# Copy static files
COPY --from=builder /usr/src/app/static /usr/local/bin/static

# Expose the port the app runs on
EXPOSE 8087

# Run the binary
CMD ["rust-chat-app"] 