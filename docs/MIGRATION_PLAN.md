# Migration Plan

## Phase 0: Audit

- Source repo cloned at `/Users/cgh/Documents/Monitor/GM-source`.
- Rust rewrite created at `/Users/cgh/Documents/Monitor/GM-rs`.
- Current source is private/proprietary, so the new personal GitHub repo should remain private unless explicitly made public later.

## Phase 1: Deterministic Core

- Port rule scoring, taxonomy, macro blending, quant indicators, feature snapshots, GBM/flow prediction, event-study calibration, and pure decision fusion.
- Lock behavior with unit tests and golden-value tests.
- Keep `decide` free of network/database/clock calls.

## Phase 2: Persistence

- Port append-only ledgers to SQLx migrations.
- Add repositories for raw events, normalized events, macro signals, price bars, feature snapshots, prediction records, decisions, rule traces, orders, broker feedback, capital transactions, and projections.
- Add replay fixtures from the Python implementation.

## Phase 3: API Parity

- Port FastAPI routers to Axum route groups.
- Generate OpenAPI schema.
- Add API contract tests against legacy fixtures.

## Phase 4: Ingestion And Workers

- Port NSE/BSE/rss/macro adapters.
- Add circuit breaker and rate-limit layers.
- Move calibration and prediction jobs into `gm-worker`.

## Phase 5: Execution

- Port paper and Zerodha adapters.
- Keep broker side effects downstream of decision/risk.
- Rebuild portfolio state strictly from fills and capital events.

## Phase 6: UI

Best engineering choice: keep a TypeScript/React dashboard until the Rust backend reaches parity. A full Rust UI is possible with Leptos, but the existing dashboard ecosystem is stronger in React for charts, drag layouts, and test tooling.

## Phase 7: Cutover

- Run old and new engines on identical event fixtures.
- Compare scores, classes, decisions, trace trees, and replay output.
- Freeze migration with a parity report before any production switch.
