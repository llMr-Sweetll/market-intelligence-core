# Debugging

## API

```bash
RUST_LOG=debug make run-api
curl http://127.0.0.1:8000/health
```

Run the API smoke test:

```bash
make smoke-api
```

If the API response changes, capture the JSON request and reproduce it through
`gm-domain` tests before changing HTTP code.

## Database

```bash
make verify-postgres
```

If migration fails:

- check `DATABASE_URL`
- if using Docker, confirm Docker is running and inspect `docker compose -f infra/docker-compose.yml logs postgres`
- if Docker is not installed, confirm `initdb`, `pg_ctl`, `createdb`, and `psql` are on PATH
- rerun `cargo run -p gm-worker -- migrate --database-url "$DATABASE_URL" --migrations migrations`

## Decision Behavior

When a decision looks wrong, inspect inputs in this order:

1. matched rules and rule score
2. macro score
3. feature signal
4. prediction signal
5. relationship modifier
6. entry price availability

No executable BUY/SELL should be emitted without an entry price.

## Logs

Useful log settings:

```bash
RUST_LOG=gm_api=debug,gm_worker=debug,gm_domain=debug,tower_http=debug
```
