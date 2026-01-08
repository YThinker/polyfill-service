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
COPY --from=builder /app/target/release/service /usr/local/bin/service

# Polyfill data (can be overridden by mounting a volume)
COPY polyfill-libraries /app/polyfill-libraries

ENV POLYFILL_BASE=/app/polyfill-libraries
ENV CACHE_DIR=/app/cache-dir
ENV PORT=8787

EXPOSE 8787

CMD ["/usr/local/bin/service"]

