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
- Payment state reports Razorpay test mode with live billing disabled.
- Payment order creation returns a deterministic test order and checkout
  signature fixture.
- Checkout signature verification succeeds for the fixture and webhook
  verification succeeds for a signed raw body.
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
runs the visible payment test-mode verification flow, and checks that the
evidence, risk gates, replay metadata, similar-event history, and missing-fact
state render in Chromium.

The browser suite also runs:

- desktop and mobile projects for responsive coverage
- primary-screen accessibility scan for WCAG 2.x A/AA serious and critical
  violations
- UI decision-flow p95 performance check; default budget:
  `UI_DECISION_P95_MAX_MS=2500`

## Performance Gates

```bash
make perf-check
```

This runs:

- `cargo test -p gm-domain --test decision_perf --all-features`
- `npm run test:e2e --prefix apps/web -- --grep p95`

The Rust decision-path check measures score plus decision generation over a
deterministic fixture. Default budget: `DECISION_P95_MAX_MICROS=25000`.

The UI p95 check measures the `/decide` request and resulting DOM update through
the browser workflow. Default budget: `UI_DECISION_P95_MAX_MS=2500`.

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
- payment API state, order, checkout verification, signed webhook, and
  bad-signature contracts
- Event Inbox API list/detail contracts and component filtering
- Decision Workbench component and browser smoke coverage
- primary-screen accessibility scan
- pure decision-path and UI decision-flow p95 performance gates

## What Still Needs Coverage

- repository read integration tests against PostgreSQL
- replay tests that rebuild projections from append-only facts
- live broker execution state-machine tests
