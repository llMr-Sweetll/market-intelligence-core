# Contributing

Market Intelligence Core is source-available under a noncommercial license.
Contributions are welcome when they improve correctness, verification,
documentation, or release quality.

## Scope

Good contributions usually fit one of these areas:

- deterministic domain math
- event-study calibration
- knowledge-base taxonomy and fixtures
- API contracts and persistence
- frontend workflows and accessibility
- provider adapters and test fixtures
- documentation and release checks

Live broker order placement is out of scope for `v0.1.0`.

## Local Setup

```bash
cp .env.example .env
make check
make run-api
```

In another terminal:

```bash
make smoke-api
```

For migration verification:

```bash
make verify-postgres
```

## Pull Requests

Before opening a pull request:

- keep changes focused on one issue or one clear workstream
- add or update tests for behavior changes
- update docs for user-facing or operational changes
- run `make check`
- include screenshots for frontend changes
- do not commit credentials, tokens, account identifiers, broker secrets, or payment secrets

## Decision Model Changes

Decision behavior must be replayable. Model changes need:

- deterministic fixtures
- before/after examples
- explanation of changed assumptions
- tests proving stable replay for the same inputs
- documentation updates in `docs/MODEL.md` when semantics change

## Frontend Changes

The interface is an operator console, not a marketing page. Prefer dense,
scannable layouts, clear state, and evidence-first presentation.

Frontend changes should verify:

- loading, empty, error, stale, and success states
- desktop and mobile layout
- keyboard reachability for primary workflows
- no text overlap or clipped buttons

