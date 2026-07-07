# Decision Model

The decision model must move beyond direct keyword thresholds. A decision should
be the result of explicit evidence, calibrated impact estimates, and risk gates.

## Invariants

- Domain math is deterministic.
- Domain math does not call networks, databases, clocks, or random generators.
- Decisions are replayable from normalized inputs.
- Executable BUY or SELL output requires an as-of price and risk context.
- Live broker execution is disabled in `v0.1.0`.

## Pipeline

```text
normalized event
  -> entity resolution
  -> event classification
  -> evidence collection
  -> impact estimation
  -> risk and confidence gates
  -> decision
  -> trace and replay record
```

## Event Classification

Each event receives:

- event class
- affected entities
- sector and geography
- time validity
- source reliability
- corroboration count
- severity
- expected transmission path

Transmission path examples:

- regulation -> sector margin -> listed firms
- conflict -> commodity supply -> energy sector -> FX and index effects
- company filing -> firm cash flow -> instrument-level repricing
- medical classification or health alert -> pharma, insurance, public-health,
  and logistics exposure

## Impact Estimation

Impact estimation should combine:

- historical event-study evidence
- current macro context
- market regime
- instrument volatility
- liquidity
- source confidence
- known cross-market relationships

The baseline finance metric is abnormal return:

```text
abnormal_return = observed_return - expected_return
```

Expected return can start with a market model:

```text
expected_return = alpha + beta * benchmark_return
```

Cumulative abnormal return:

```text
CAR[t1,t2] = sum(abnormal_return[t]) for t in event window
```

The release model can begin with deterministic CAR fixtures and expand toward
calibrated estimates as ingestion improves.

## Confidence

Confidence is not just keyword match strength. It should combine:

- event-class certainty
- entity-resolution certainty
- source reliability
- corroboration
- recency
- historical sample size
- market-data completeness
- model calibration quality

Confidence should decrease when required facts are missing.

## Decision Policy

For each candidate action, estimate:

- expected return
- downside risk
- liquidity penalty
- volatility penalty
- transaction-cost penalty
- confidence multiplier
- missing-fact penalty

Then choose the highest expected utility among:

- HOLD
- BUY
- SELL
- PAPER

`PAPER` is a simulation-only action used for validating broker and execution
flows without live orders.

## Output Contract

Every decision includes:

- decision ID
- model version
- input hash
- parent event ID and version
- action
- confidence
- expected return estimate
- downside estimate
- position sizing recommendation, if any
- missing facts
- evidence trace
- explanation
- execution readiness

The current implementation writes this information directly into the domain
`Decision` response. The structured explanation includes pipeline stages,
entity resolution, evidence items, impact estimate, gates, bounded utilities for
BUY/SELL/HOLD/PAPER, missing facts, and a summary.

## Release Gates

`v0.1.0` should prove:

- deterministic replay
- meaningful HOLD when evidence is insufficient
- BUY/SELL only with explicit as-of facts
- event-study calculations on fixtures
- confidence changes when evidence quality changes
- no live broker order path is enabled
