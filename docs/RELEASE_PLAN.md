# Release Plan

Target milestone: `v0.1.0-mv`

## Release Definition

The minimum viable release is a public, source-available system that proves:

- deterministic backend decision behavior
- evidence-backed model direction
- operator UI foundation
- local API and frontend working together
- migration verification
- fixture integrations
- Razorpay test-mode payment path
- no live broker order placement
- clear contribution and security process

## Workstreams

1. Public readiness
2. UI/UX foundation
3. Decision Workbench
4. Knowledge base
5. Evidence-backed model
6. API persistence and contracts
7. Provider adapters and fixtures
8. Razorpay test mode
9. Test and performance gates
10. Release automation

## PR Sequence

1. Public readiness, licensing, contribution docs, UI/model/release plans.
2. Web app shell and design system.
3. Decision Workbench connected to local API.
4. Knowledge ontology and fixtures.
5. Evidence-backed model and event-study calculations.
6. API persistence, `/ready`, `/version`, and OpenAPI.
7. Provider adapter framework and fixture providers.
8. Razorpay test-mode payment flow.
9. Browser, accessibility, API contract, and performance checks.
10. Docker packaging, tag release workflow, and public switch.

## Release Gates

Before making the repository public:

- source-available license is present
- README clearly states license terms
- contribution and security docs are present
- issue templates and PR template are present
- direct tool-attribution and secret scans pass
- local `make check` passes
- API smoke test passes
- migration verifier passes

Before tagging `v0.1.0`:

- frontend builds and tests pass
- Playwright smoke passes
- API contract tests pass
- Docker image builds
- release notes are written
- tag workflow creates a draft GitHub release artifact
- the combined API and web container serves `/health` and `/`
- live broker order placement remains disabled

## GitHub Issues

The `v0.1.0-mv` milestone tracks implementation issues. Each PR should close or
advance one small group of issues rather than mixing unrelated work.
