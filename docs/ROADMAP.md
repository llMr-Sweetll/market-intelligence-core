# Roadmap

This project is a standalone market intelligence core. The near-term goal is to
keep the engine small, deterministic, and easy to verify before adding more data
sources or execution adapters.

## Implemented

- Deterministic event scoring with keyword rules and event classification.
- Bounded macro-context blending by sector.
- Technical indicators and as-of feature snapshots.
- GBM plus flow-adjusted prediction records.
- Deterministic decision generation with as-of price injection.
- PostgreSQL migration support for append-only ledgers.
- Axum HTTP API for health, rules, scoring, decisions, features, prediction,
  and macro context.
- Local smoke scripts for API and PostgreSQL migration checks.

## Next

- Persist API decisions and rule traces through repository interfaces.
- Add fixture-based HTTP contract tests.
- Add market-data ingestion jobs with source-level circuit breakers.
- Add replay tests that rebuild derived projections from append-only facts.
- Add paper execution after risk checks are fully covered by tests.
- Add a release image workflow only when publishing is needed.

## Operating Rule

Do not add live network calls, clocks, random identifiers, or database writes to
the domain scoring path. External inputs must be collected upstream and passed in
as explicit facts.
