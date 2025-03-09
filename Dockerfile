# Stage 1: Build the application
FROM --platform=linux/amd64 rust:1.85-slim-bookworm as builder

WORKDIR /usr/src/kairei
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY kairei-core ./kairei-core
COPY kairei-http ./kairei-http
COPY kairei-cli  ./kairei-cli

RUN cargo build --release --bin kairei-http

# Stage 2: Create a minimal runtime image
FROM --platform=linux/amd64 debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Create a non-root user for running the application
RUN useradd -ms /bin/bash kairei
USER kairei

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/kairei/target/release/kairei-http /app/

# Expose the API port
EXPOSE 8080

ENV RUST_LOG=info

# Run the server
CMD ["/app/kairei-http", "--host", "0.0.0.0", "--port", "8080"]
