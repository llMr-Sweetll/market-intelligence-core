# Architecture

Market Intelligence Core is split into a pure domain layer and thin operational
layers around it. This keeps decision behavior testable and replayable.

## Stack

- Rust 2024 workspace.
- Axum and Tokio for HTTP.
- SQLx and PostgreSQL for append-only facts.
- Docker Compose or local PostgreSQL tools for migration verification.
- Minimal CI for format, lint, tests, and dependency audit.

## System Shape

```mermaid
flowchart LR
    Source["Market / Macro / Broker Inputs"] --> Normalize["Normalize"]
    Normalize --> Event["NormalizedEvent"]
    Event --> Domain["Pure Domain Core"]
    Facts["As-Of Facts: PriceBar / FeatureVector / PredictionRecord / MacroContext"] --> Domain
    Domain --> Decision["Decision"]
    Decision --> Risk["Risk Check"]
    Risk --> Execution["Execution Adapter"]
    API["HTTP API"] --> Domain
    Worker["Worker Commands"] --> Storage["PostgreSQL"]
```

## Boundaries

- `gm-domain` contains pure functions and serializable data types.
- `gm-api` converts HTTP requests into domain inputs and returns domain outputs.
- `gm-persistence` owns database connection and migration helpers.
- `gm-worker` owns operational commands and future batch jobs.

## Invariants

- Scoring and decision functions do not call networks or databases.
- Scoring and decision functions do not read wall-clock time.
- Decision IDs are deterministic for the same event, facts, score, and action.
- Price is always an injected as-of fact.
- Derived facts are written upstream, then passed into the decision path.
- Database tables that represent facts are append-only by design.

## Minimal Runtime

Development requires Rust and Cargo. PostgreSQL is only required for migration
and persistence checks; the domain and API smoke tests can run without a
database.
