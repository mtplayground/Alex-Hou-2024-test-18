FROM rust:1.90.0-bookworm AS frontend-builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk --locked --version 0.21.14

WORKDIR /app
COPY . .

WORKDIR /app/frontend
RUN trunk build --config Trunk.toml --release

FROM rust:1.90.0-bookworm AS backend-builder

WORKDIR /app
COPY . .

RUN cargo build --locked --release -p backend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 appuser

WORKDIR /app

COPY --from=backend-builder /app/target/release/backend /app/backend
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

ENV BIND_ADDR=0.0.0.0:8080
ENV RUST_LOG=backend=info,tower_http=info
ENV STATIC_DIR=/app/frontend/dist

USER appuser

EXPOSE 8080

CMD ["/app/backend"]
