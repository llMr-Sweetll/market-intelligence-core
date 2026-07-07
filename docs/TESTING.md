# Testing

Testing focuses on proving that the domain math is deterministic and that the
runtime paths work locally.

## Local Gates

```bash
make check
```

This runs:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit --deny warnings
```

## API Smoke Test

Start the API:

```bash
make run-api
```

In another terminal:

```bash
make smoke-api
```

The smoke test verifies:

- `GET /health` returns `ok`.
- `GET /ready` returns readiness and persistence status.
- `GET /version` returns service and model versions.
- `GET /openapi.json` exposes the contract paths.
- `POST /decide` returns an executable BUY for a strong earnings event with an
  injected price.
- Quantity, target, stop, and score are stable.

## Browser Smoke Test

```bash
make web-e2e
```

This starts the Rust API without persistence, starts the Vite app with
`VITE_API_BASE_URL` pointed at that API, filters the Event Inbox fixture feed,
checks selected event detail metadata, submits the Decision Workbench fixture,
and checks that the evidence, risk gates, replay metadata, similar-event
history, and missing-fact state render in Chromium.

## PostgreSQL Migration Check

```bash
make verify-postgres
```

This starts local PostgreSQL through Docker Compose when Docker is available.
When Docker is not installed and `DATABASE_URL` is not set, it starts a
temporary local PostgreSQL cluster, applies SQLx migrations through `gm-worker`,
and checks that the expected schema exists.

## Domain Coverage

Current unit tests cover:

- keyword word-boundary matching
- event classification
- macro normalization and sector weighting
- indicators and feature as-of cutoffs
- GBM quantile determinism
- flow adjustment caps
- event-study forward-return math
- BUY/SELL/HOLD decision behavior
- deterministic decision IDs
- provider adapter fixtures for market data, events, filings, entity mapping,
  payments, and paper execution
- Event Inbox API list/detail contracts and component filtering
- Decision Workbench component and browser smoke coverage

## What Still Needs Coverage

- repository read integration tests against PostgreSQL
- replay tests that rebuild projections from append-only facts
- live broker execution state-machine tests
