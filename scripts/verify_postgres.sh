#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="${COMPOSE_FILE:-infra/docker-compose.yml}"
MIGRATIONS="${MIGRATIONS:-migrations}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Required command not found: $1" >&2
    exit 127
  fi
}

run_migrations() {
  cargo run -p gm-worker -- migrate --database-url "$1" --migrations "$MIGRATIONS"
}

assert_schema_with_psql() {
  local database_url="$1"
  local table_count
  table_count="$(psql "$database_url" -tAc \
    "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public';")"

  if [ "$table_count" -lt 8 ]; then
    echo "Expected migrated schema to contain at least 8 tables, found $table_count" >&2
    exit 1
  fi

  echo "PostgreSQL migration check passed with $table_count public tables"
}

if command -v docker >/dev/null 2>&1; then
  DATABASE_URL="${DATABASE_URL:-postgres://gm:gm@localhost:5432/gm}"

  docker compose -f "$COMPOSE_FILE" up -d postgres

  for _ in $(seq 1 30); do
    if docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U gm -d gm >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done

  docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U gm -d gm >/dev/null
  run_migrations "$DATABASE_URL"

  table_count="$(
    docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U gm -d gm -tAc \
    "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public';"
  )"

  if [ "$table_count" -lt 8 ]; then
    echo "Expected migrated schema to contain at least 8 tables, found $table_count" >&2
    exit 1
  fi

  echo "PostgreSQL migration check passed with $table_count public tables"
  exit 0
fi

if [ -n "${DATABASE_URL:-}" ]; then
  require_command psql
  run_migrations "$DATABASE_URL"
  assert_schema_with_psql "$DATABASE_URL"
  exit 0
fi

require_command initdb
require_command pg_ctl
require_command createdb
require_command psql

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/market-intelligence-core-postgres.XXXXXX")"
pgdata="$tmp_dir/data"
pgport="${PGPORT:-55432}"
database_url="postgres://gm@127.0.0.1:${pgport}/gm"

cleanup() {
  if [ -d "$pgdata" ]; then
    pg_ctl -D "$pgdata" stop -m fast >/dev/null 2>&1 || true
  fi
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

initdb -D "$pgdata" --username=gm --auth=trust >/dev/null
pg_ctl -D "$pgdata" -o "-h 127.0.0.1 -p $pgport" -l "$tmp_dir/postgres.log" start >/dev/null
createdb -h 127.0.0.1 -p "$pgport" -U gm gm

run_migrations "$database_url"
assert_schema_with_psql "$database_url"
