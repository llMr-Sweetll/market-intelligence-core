# Release Notes

## v0.1.0

Date: TBD

## Summary

- Deterministic market event scoring and decision API.
- Operator console with Event Inbox, Decision Workbench, Knowledge, Market
  Impact, Integrations, and Payments surfaces.
- Source-available, noncommercial licensing.
- Live broker order placement remains unavailable.

## Verification Before Publishing

- [ ] Main CI passed.
- [ ] `make check-all` passed.
- [ ] `make web-e2e` passed.
- [ ] `make perf-check` passed.
- [ ] `make verify-postgres` passed.
- [ ] `make smoke-api` passed against a local API.
- [ ] `make docker-build` passed.
- [ ] Container smoke test passed for `/health` and `/`.

## Included Artifacts

- `gm-api` release binary.
- Built web assets.
- SQL migrations.
- License and README.
- Docker image build recipe.

## Operational Notes

- Set `WEB_ASSETS_DIR` to serve the built web app from `gm-api`.
- Set `DATABASE_URL` to enable PostgreSQL persistence and startup migrations.
- Production secrets must be provided through the runtime environment.
- Razorpay and broker integrations remain in test, mock, read-only, or paper
  modes unless explicitly configured outside this release.

## Known Limits

- Live ingestion providers are represented by fixtures and adapters.
- Live broker execution is blocked.
- Payment flow is planned for test-mode verification before paid access gates.
