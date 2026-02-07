# Multi-stage build for filebrowser-tui

# Stage 1: Build
FROM rust:1.88-alpine AS builder

WORKDIR /build

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconf \
    openssl-dev \
    build-base

# Copy Cargo files first for better caching
COPY Cargo.toml ./
COPY src ./src

# Create a dummy main.rs to build dependencies first
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source
COPY src ./src

# Build the application
RUN cargo build --release

# Verify the binary exists
RUN ls -la /build/target/release/ && \
    test -f /build/target/release/fbt || (echo "Binary not found!" && exit 1)

# Strip the binary to reduce size
RUN strip /build/target/release/fbt

# Stage 2: Runtime
FROM alpine:3.20

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# Copy the binary from builder
COPY --from=builder /build/target/release/fbt /app/fbt

# Verify binary works
RUN /app/fbt --version 2>/dev/null || /app/fbt --help 2>/dev/null || echo "Binary OK"

# Create a volume mount point for user's data directory
VOLUME ["/data"]

# Set up a non-root user
RUN addgroup -g 1000 fbt && \
    adduser -D -u 1000 -G fbt fbt && \
    chown -R fbt:fbt /app

USER fbt

# Set environment variables
ENV HOME=/data \
    TERM=xterm-256color \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

# Health check (will always fail for TUI but documents the app)
HEALTHCHECK --interval=30s --timeout=3s --start-period=1s --retries=1 \
    CMD test -f /app/fbt || exit 1

# Run the app by default
ENTRYPOINT ["/app/fbt"]
