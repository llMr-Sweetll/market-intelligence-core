# API Contract

The HTTP API is a thin runtime wrapper over the deterministic domain core. The
decision path accepts explicit input facts and does not fetch data on its own.

## Operational Endpoints

- `GET /health` returns a liveness response for the API process.
- `GET /ready` reports service readiness and optional PostgreSQL status.
- `GET /version` returns the API version and current model version.
- `GET /openapi.json` returns the OpenAPI 3.1 contract.

When `DATABASE_URL` is unset, `/ready` still returns ready because persistence is
optional for local domain and smoke-test workflows. When `DATABASE_URL` is set,
the API connects to PostgreSQL at startup, applies migrations by default, and
`/ready` verifies that the database remains reachable.

## Decision Endpoint

`POST /decide` accepts:

- `event`: a normalized event with a stable `event_id` and `version`.
- `facts`: optional as-of facts such as macro context, entry price, exchange,
  features, prediction, and knowledge-graph modifier.

If persistence is configured, a successful decision writes:

- the normalized event
- the score projection on the event row
- matched rule traces
- the decision
- the replay input snapshot and input hash

If any configured persistence write fails, the endpoint returns `500` instead of
returning an unaudited decision.

## Local Contract Check

```bash
make run-api
make smoke-api
```

The smoke script checks `/health`, `/ready`, `/version`, `/openapi.json`, and the
deterministic BUY fixture on `/decide`.
