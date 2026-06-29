# Multi-stage Docker build for mermaid-cli
# Stage 1: Build the binary
FROM rust:1.81-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/

# Build with all features
RUN cargo build --release --features "json,png" && \
    strip target/release/mermaid-cli

# Stage 2: Minimal runtime image
FROM alpine:3.20

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/mermaid-cli /usr/local/bin/mermaid-cli

ENTRYPOINT ["mermaid-cli"]
CMD ["--help"]
