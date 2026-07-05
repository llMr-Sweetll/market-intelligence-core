.PHONY: fmt clippy test check run-api run-worker docker-up docker-down migrate

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

check: fmt clippy test

run-api:
	cargo run -p gm-api -- --host $${GM_HOST:-127.0.0.1} --port $${GM_PORT:-8000}

run-worker:
	cargo run -p gm-worker -- check

docker-up:
	docker compose -f infra/docker-compose.yml up -d

docker-down:
	docker compose -f infra/docker-compose.yml down

migrate:
	sqlx migrate run --source migrations
