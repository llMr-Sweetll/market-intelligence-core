FROM rust:1.96-bookworm AS builder
WORKDIR /app

COPY Cargo.toml rust-toolchain.toml ./
COPY crates ./crates
RUN cargo build --release -p gm-api

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gm-api /usr/local/bin/gm-api
EXPOSE 8000
CMD ["gm-api", "--host", "0.0.0.0", "--port", "8000"]
