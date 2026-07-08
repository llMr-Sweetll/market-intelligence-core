# Public Readiness Checklist

The repository should stay public only while this checklist stays true on
`main`.

## Required

- [x] License is source-available and noncommercial.
- [x] README states the project is not open-source licensed.
- [x] Contribution guide exists.
- [x] Security policy exists.
- [x] Support policy exists.
- [x] Pull request template exists.
- [x] Issue templates exist.
- [x] CODEOWNERS exists.
- [x] Release plan exists.
- [x] No real credentials, tokens, account IDs, or secrets are present.
- [x] No direct tool-attribution references are present in tracked files.
- [x] No stale private-only wording remains.
- [x] CI passes.
- [x] API smoke test passes.
- [x] Migration verification passes.
- [x] API readiness endpoint and OpenAPI contract exist.
- [x] Release checklist exists.
- [x] Docker image build is covered by CI.

## Public Visibility Notes

When the repository is public:

- code is visible to anyone
- forks can be created
- Actions logs are visible
- public issues and pull requests are visible

If a future change breaks licensing, public-facing documentation, CI, smoke
tests, or secret hygiene, fix it before tagging a release.
