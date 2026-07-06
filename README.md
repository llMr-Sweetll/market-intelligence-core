# Market Intelligence Core

Market Intelligence Core is a Rust service for turning market events and
as-of market facts into deterministic, explainable trading decisions.

The core rule is simple: the decision path never calls live services, reads the
clock, or generates random identifiers. Every decision is computed from explicit
inputs: a normalized event, rule definitions, macro context, price facts,
technical features, prediction records, and bounded relationship modifiers.

License: proprietary internal-use only. This repository is not open source.

## What Works Now

- Deterministic event scoring and event classification.
- BUY/SELL/HOLD decision generation from frozen as-of facts.
- Technical indicators: returns, SMA, EMA, RSI, ATR, volatility, drawdown,
  z-score, and beta.
- Feature-vector calculation with no look-ahead beyond the supplied as-of date.
- GBM plus flow-adjusted return prediction with deterministic quantiles.
- Event-study calibration helpers for forward and abnormal returns.
- Axum HTTP API for scoring, decisions, features, prediction, and macro context.
- SQLx/PostgreSQL migrations for append-only market, feature, prediction,
  decision, and trace tables.
- Local scripts for API smoke testing and PostgreSQL migration verification.

## Workspace

| Crate | Purpose |
| --- | --- |
| `gm-domain` | Pure domain logic: rules, classification, scoring, indicators, features, predictions, event studies, risk checks. |
| `gm-api` | HTTP API over the domain core. |
| `gm-persistence` | PostgreSQL repository and migration helpers. |
| `gm-worker` | Operational commands such as migration checks and future batch jobs. |

## Quick Start

```bash
cp .env.example .env
make check
make run-api
```

In another terminal:

```bash
make smoke-api
```

For PostgreSQL migration verification:

```bash
make verify-postgres
```

## API

- `GET /health`
- `GET /rules`
- `POST /score`
- `POST /decide`
- `POST /quant/features`
- `POST /predict/gbm`
- `POST /macro/context`

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Algorithm](docs/ALGORITHM.md)
- [Testing](docs/TESTING.md)
- [Debugging](docs/DEBUGGING.md)
- [Roadmap](docs/ROADMAP.md)
