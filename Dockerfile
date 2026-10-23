FROM rust:1.94-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /usr/sbin/nologin aeroledger \
    && mkdir -p /app/uploads \
    && chown -R aeroledger:aeroledger /app

COPY --from=builder /app/target/release/areo_ledger /usr/local/bin/areo_ledger
COPY --from=builder /app/target/release/migrate /usr/local/bin/areo_ledger_migrate

USER aeroledger

ENV APP_ENV=production \
    RUN_MIGRATIONS=false \
    HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    PAYMENT_VERIFICATION_MODE=live

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -fsS "http://127.0.0.1:${PORT:-8080}/api/health" || exit 1

CMD ["areo_ledger"]
