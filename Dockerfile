# Stage 1: Build the application
FROM rust:1.77-slim-bookworm as builder

WORKDIR /usr/src/kairei
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY kairei-core/Cargo.toml ./kairei-core/
COPY kairei-http/Cargo.toml ./kairei-http/

# Create dummy source files for dependency caching
RUN mkdir -p src kairei-core/src kairei-http/src && \
    touch src/lib.rs kairei-core/src/lib.rs kairei-http/src/lib.rs && \
    echo "fn main() {}" > kairei-http/src/bin/kairei-http.rs && \
    cargo build --release --bin kairei-http

# Copy the actual source code
COPY . .

# Force rebuild with actual source code
RUN touch kairei-core/src/lib.rs kairei-http/src/lib.rs kairei-http/src/bin/kairei-http.rs && \
    cargo build --release --bin kairei-http

# Stage 2: Create a minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Create a non-root user for running the application
RUN useradd -ms /bin/bash kairei
USER kairei

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/kairei/target/release/kairei-http /app/

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Expose the API port
EXPOSE 3000

ENV RUST_LOG=info

# Run the server
CMD ["/app/kairei-http", "--host", "0.0.0.0", "--port", "3000"]