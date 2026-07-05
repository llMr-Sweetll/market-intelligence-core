# Testing

## Test Pyramid

- Unit tests in `gm-domain` for every pure algorithm.
- Repository tests in `gm-persistence` against disposable Postgres.
- HTTP contract tests for `gm-api`.
- Replay parity tests using frozen event/fact fixtures from the Python repo.
- End-to-end paper-trading tests after execution adapters are ported.

## Required Gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
```

## Determinism Tests

- Same event + same registry returns identical score output.
- Same price bars + same as-of date returns identical features.
- Same prices + same flow input returns identical prediction.
- Same decision input returns byte-equivalent decision JSON.
- Replay must never touch production storage.

## Future Golden Fixtures

Use the Python repo tests as behavior fixtures, then add stricter Rust fixtures:

- classification edge cases
- keyword word-boundary cases
- CAR calibration samples
- RSI/ATR/EMA golden values
- GBM quantiles
- broker state machine transitions
