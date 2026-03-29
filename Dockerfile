FROM rust:1.89 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/areo_ledger /usr/local/bin/areo_ledger
COPY migrations ./migrations
COPY .env.example ./.env.example
EXPOSE 8080
CMD ["/usr/local/bin/areo_ledger"]
