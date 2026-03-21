# syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS chef
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libsqlite3-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json ./recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=splitsmart-cargo-registry \
    --mount=type=cache,target=/usr/local/cargo/git/db,id=splitsmart-cargo-git \
    --mount=type=cache,target=/app/target,id=splitsmart-target \
    cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=splitsmart-cargo-registry \
    --mount=type=cache,target=/usr/local/cargo/git/db,id=splitsmart-cargo-git \
    --mount=type=cache,target=/app/target,id=splitsmart-target \
    cargo build --release --bin tg_split_smart_bot \
    && mkdir -p /app/dist \
    && cp /app/target/release/tg_split_smart_bot /app/dist/tg_split_smart_bot

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libsqlite3-0 tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/data

COPY --from=builder /app/dist/tg_split_smart_bot /usr/local/bin/tg_split_smart_bot
ENV RUST_LOG=info
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/tg_split_smart_bot"]
