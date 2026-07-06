# Algorithm

The engine converts a normalized event and explicit as-of facts into a decision.
It is deterministic: same input, same output.

## Inputs

- `NormalizedEvent`: headline, body, symbol, sector, source, and timestamp.
- `RuleRegistry`: deterministic keyword rules with weights and confidence.
- `MacroContext`: normalized market and capital-flow context.
- Optional `FeatureVector`: technical features calculated as of the event date.
- Optional `PredictionRecord`: forward return forecast as of the event date.
- Optional relationship modifier: bounded contextual adjustment.
- Optional entry price: required for executable BUY/SELL decisions.

## Rule Score

Each matched rule contributes:

```text
contribution = weight * confidence
```

The rule score is:

```text
rule_score = clamp(sum(contributions), -1, 1)
```

Example:

```text
EARNINGS_BEAT = 0.8 * 0.9 = 0.72
```

With the default `0.70` action threshold and a supplied entry price, that is an
actionable BUY.

## Macro Context

Percentage signals are normalized as:

```text
normalized_pct = clamp(change_percent / 5, -1, 1)
```

FII flow is normalized as:

```text
fii_signal = clamp(fii_net_cr / 2000, -1, 1)
```

The sector macro score is a weighted blend of normalized factors.

## Feature Signal

The feature signal is deliberately bounded so technical factors can confirm or
soften an event signal without dominating it:

```text
momentum_component = avg(momentum_1m, momentum_3m) * 2
rsi_component = +0.05 when RSI < 30, -0.05 when RSI > 70
zscore_component = +0.05 when z < -2, -0.05 when z > 2
feature_signal = clamp(sum, -0.20, 0.20)
```

## Prediction Signal

GBM calibration uses daily log returns. Capital-flow pressure nudges drift:

```text
flow_adjustment = clamp(scale * normalized_flow, -cap, cap)
adjusted_mu = mu + flow_adjustment
```

The expected return is bounded before decision fusion:

```text
prediction_signal = clamp(expected_return * 3, -0.20, 0.20)
```

## Decision Fusion

```text
combined = clamp(
  rule_score
  + 0.30 * macro_score
  + feature_signal
  + prediction_signal
  + relationship_modifier,
  -1,
  1
)
```

Default actions:

```text
BUY  when combined >=  0.70
SELL when combined <= -0.70
HOLD otherwise
```

Position sizing:

```text
5% notional when abs(combined) >= 0.85
2% notional for other executable BUY/SELL decisions
0% for HOLD
```

Targets and stops use ATR when available:

```text
BUY target = entry + max(2 * ATR, 3% entry)
BUY stop   = entry - max(1.2 * ATR, 2% entry)
SELL target/stop mirror the direction
```

If no entry price is provided, the system returns HOLD even when the signal
crosses the action threshold. This prevents accidental executable decisions from
non-executable inputs.
