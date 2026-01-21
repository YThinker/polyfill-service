FROM rust:1.74 as builder

WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
COPY library/Cargo.toml library/Cargo.toml
COPY service/Cargo.toml service/Cargo.toml
RUN mkdir library/src service/src && \
    echo "// placeholder" > library/src/lib.rs && \
    echo "// placeholder" > service/src/lib.rs && \
    cargo fetch

# Build
COPY . .
RUN cargo build -p service --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/service /polyfill-service/service

# Polyfill data (can be overridden by mounting a volume)
COPY polyfill-libraries /polyfill-service/polyfill-libraries

ENV POLYFILL_BASE=/polyfill-service/polyfill-libraries
ENV CACHE_DIR=/polyfill-service/cache-dir
ENV PORT=8787

EXPOSE 8787

# Clear cache directory on each container start
CMD ["sh", "-c", "rm -rf ${CACHE_DIR}/* 2>/dev/null || true && /polyfill-service/service"]