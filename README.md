# GM-rs

Rust rewrite of GM, an event-driven market intelligence platform for Indian equities.

This repo starts from the existing GM architecture but fixes the main replay gap immediately: the decision path is pure. It never reaches out to live prices or mutable services. Decisions are computed from normalized events plus frozen as-of facts: macro context, price bars/features, prediction records, and knowledge-graph modifiers.

License: proprietary internal-use only. This is not open source.

## Workspace

| Crate | Purpose |
| --- | --- |
| `gm-domain` | Deterministic business core: rules, classification, scoring, quant features, prediction, event-study calibration, decision fusion. |
| `gm-api` | Axum HTTP API over the domain core. |
| `gm-persistence` | SQLx/Postgres repositories and migrations for append-only ledgers. |
| `gm-worker` | Batch/offline jobs shell for ingestion, calibration, prediction, and replay jobs. |

## Quick Start

```bash
cp .env.example .env
make docker-up
make test
make run-api
```

API:

- `GET /health`
- `GET /rules`
- `POST /score`
- `POST /decide`
- `POST /quant/features`
- `POST /predict/gbm`

## Design Docs

- [Architecture](docs/ARCHITECTURE.md)
- [Algorithm](docs/ALGORITHM.md)
- [Migration Plan](docs/MIGRATION_PLAN.md)
- [Testing](docs/TESTING.md)
- [Debugging](docs/DEBUGGING.md)

## Current Status

This is the first Rust cut: a compiling, tested domain and service skeleton with CI, release/CD, migrations, Docker, proprietary licensing, dependency audit, and docs. It is not yet full feature parity with the Python/Next implementation. The intended next work is to port ingestion adapters, execution adapters, replay parity, and then the dashboard.
