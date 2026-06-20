FROM rust:1.96.0-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libpq5 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system app \
    && useradd --system --gid app --home-dir /app --shell /usr/sbin/nologin app

WORKDIR /app

COPY --from=builder /app/target/release/rust-simple-restapi-templ /usr/local/bin/rust-simple-restapi-templ

ENV APP_ENV=production \
    RUST_LOG=info \
    APP_HOST=0.0.0.0 \
    APP_PORT=8089

EXPOSE 8089

USER app

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8089/api/v1/health/live || exit 1

CMD ["rust-simple-restapi-templ"]
