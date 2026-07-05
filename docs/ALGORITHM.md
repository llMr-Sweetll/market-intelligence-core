# Algorithm

## Inputs

- A normalized corporate, macro, or geopolitical event.
- A deterministic rule registry.
- Frozen macro context.
- Frozen price/feature/prediction/KG facts as of the event timestamp.

## Event Score

Each rule returns:

```text
contribution = weight * confidence
```

Only matched rules contribute. The event score is:

```text
rule_score = clamp(sum(contributions), -1, 1)
```

The event class is assigned by deterministic keyword taxonomy scoring.

## Macro Context

Percentage factors are normalized as:

```text
normalized_pct = clamp(change_percent / 5, -1, 1)
```

FII flow is normalized as:

```text
fii_signal = clamp(fii_net_cr / 2000, -1, 1)
```

The sector macro score is the weighted blend for the event sector.

## Feature Signal

The first Rust implementation uses a bounded technical signal:

```text
momentum_component = avg(momentum_1m, momentum_3m) * 2
rsi_component = +0.05 when RSI < 30, -0.05 when RSI > 70
zscore_component = +0.05 when z < -2, -0.05 when z > 2
feature_signal = clamp(sum, -0.20, 0.20)
```

## Prediction Signal

GBM calibration computes daily log-return mean and standard deviation. Flow nudges drift:

```text
flow_adjustment = clamp(scale * normalized_flow, -cap, cap)
adjusted_mu = mu + flow_adjustment
```

The expected return over the horizon becomes a bounded decision input:

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
  + kg_modifier,
  -1,
  1
)
```

Thresholds:

```text
BUY  when combined >=  0.75
SELL when combined <= -0.75
HOLD otherwise
```

Position size is 5% for very high confidence (`abs(score) >= 0.85`), else 2%.

Targets/stops use ATR when available, with percentage fallbacks:

```text
BUY target = entry + max(2 * ATR, 3% entry)
BUY stop   = entry - max(1.2 * ATR, 2% entry)
SELL target/stop mirror the direction
```

No price fact means no executable trade. The decision remains deterministic and returns HOLD with an explanatory thesis.
