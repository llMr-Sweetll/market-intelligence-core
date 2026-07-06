.PHONY: fmt fmt-check clippy test audit check web-install web-build web-test web-check check-all run-api run-worker docker-up docker-down migrate smoke-api verify-postgres

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

audit:
	cargo audit --deny warnings

check: fmt-check clippy test audit

web-install:
	npm ci --prefix apps/web

web-build:
	npm run build --prefix apps/web

web-test:
	npm run test --prefix apps/web

web-check:
	npm run check --prefix apps/web

check-all: check web-check

run-api:
	cargo run -p gm-api -- --host $${GM_HOST:-127.0.0.1} --port $${GM_PORT:-8000}

run-worker:
	cargo run -p gm-worker -- check

docker-up:
	docker compose -f infra/docker-compose.yml up -d

docker-down:
	docker compose -f infra/docker-compose.yml down

migrate:
	cargo run -p gm-worker -- migrate --database-url $${DATABASE_URL:-postgres://gm:gm@localhost:5432/gm} --migrations migrations

smoke-api:
	scripts/smoke_api.sh

verify-postgres:
	scripts/verify_postgres.sh
