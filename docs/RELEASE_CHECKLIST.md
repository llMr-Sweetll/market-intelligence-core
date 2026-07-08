# Release Checklist

Use this checklist before creating a `v0.1.0` tag.

## Repository

- [ ] `main` CI is green.
- [ ] Public readiness checklist remains green.
- [ ] License terms are present and visible from the README.
- [ ] Contribution, support, and security docs are current.
- [ ] Issue templates and pull request template are present.
- [ ] No real credentials, tokens, account IDs, or secrets are present.
- [ ] No direct tool-attribution references are present in tracked files.
- [ ] Release notes are updated with the real release date and verification
  results.

## Local Verification

- [ ] `make check-all`
- [ ] `make web-e2e`
- [ ] `make perf-check`
- [ ] `make verify-postgres`
- [ ] `make smoke-api` against a local API process.
- [ ] `make docker-build`
- [ ] Container smoke test: `GET /health` returns `200`.
- [ ] Container smoke test: `GET /` returns the built web app.

## Runtime

- [ ] `DATABASE_URL` is configured for persistent environments.
- [ ] `GM_MIGRATIONS` points at the packaged migrations directory.
- [ ] `WEB_ASSETS_DIR` points at the packaged web assets directory.
- [ ] Production secrets are supplied only through runtime environment or secret
  storage.
- [ ] Live broker order placement is disabled.
- [ ] Payment behavior is limited to test-mode verification until paid access
  gates are explicitly enabled.

## GitHub Release

- [ ] Create a semantic version tag.
- [ ] Confirm the release workflow builds the binary, web assets, Docker image,
  and release archive.
- [ ] Review the generated draft release.
- [ ] Publish only after checks and artifacts are correct.
