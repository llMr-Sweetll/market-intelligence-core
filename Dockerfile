FROM node:24-bookworm-slim AS web-builder
WORKDIR /workspace/apps/web
COPY apps/web/package.json apps/web/package-lock.json ./
RUN npm ci
COPY apps/web ./
RUN npm run build

FROM rust:1.96-bookworm AS api-builder
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --locked --release -p gm-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /app --shell /usr/sbin/nologin gm
WORKDIR /app
COPY --from=api-builder /workspace/target/release/gm-api /usr/local/bin/gm-api
COPY --from=web-builder /workspace/apps/web/dist /app/web
COPY migrations /app/migrations
ENV GM_HOST=0.0.0.0 \
    GM_PORT=8000 \
    GM_MIGRATIONS=/app/migrations \
    WEB_ASSETS_DIR=/app/web
USER gm
EXPOSE 8000
CMD ["gm-api"]
