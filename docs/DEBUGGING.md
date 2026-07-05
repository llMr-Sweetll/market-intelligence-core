# Debugging

## Local Checks

```bash
RUST_LOG=debug make run-api
curl http://localhost:8000/health
```

## Useful Environment

- `RUST_LOG=gm_api=debug,gm_domain=debug,tower_http=debug`
- `DATABASE_URL=postgres://gm:gm@localhost:5432/gm`
- `REDIS_URL=redis://localhost:6379/0`

## Failure Triage

- If scoring changes, inspect rule contributions before decision fusion.
- If replay diverges, compare frozen as-of facts before comparing final decisions.
- If API output changes, serialize the domain input and run the same input through `gm-domain` unit tests.
- If database state diverges, rebuild projections from append-only ledgers and diff projection rows.

## Observability Roadmap

- Add Prometheus metrics for ingestion lag, scoring latency, decision counts, rejected risk checks, broker acknowledgements, and replay parity failures.
- Add structured audit IDs across raw event, normalized event, decision, order, and fill.
