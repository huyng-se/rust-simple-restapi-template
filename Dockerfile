FROM rust:1.91-bookworm AS builder

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

WORKDIR /app

COPY --from=builder /app/target/release/rust-simple-restapi-templ /usr/local/bin/rust-simple-restapi-templ

ENV APP_ENV=production \
    RUST_LOG=info \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080

EXPOSE 8080

CMD ["rust-simple-restapi-templ"]
