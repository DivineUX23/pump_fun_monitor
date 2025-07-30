# Multi-stage build for optimal image size
FROM rust:1.70-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release && rm -rf src target/release/deps/pump_fun_monitor_corrected*

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r pumpfun && useradd -r -g pumpfun -s /bin/false pumpfun

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/pump-fun-monitor /usr/local/bin/pump-fun-monitor

# Copy example configuration
COPY .env.example /app/.env.example

# Set ownership
RUN chown -R pumpfun:pumpfun /app

# Switch to non-root user
USER pumpfun

# Expose WebSocket port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["pump-fun-monitor"]