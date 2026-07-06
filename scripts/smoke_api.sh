#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8000}"

health="$(curl -fsS "$BASE_URL/health")"
python3 - "$health" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["status"] == "ok", payload
assert payload["service"] == "gm-api", payload
PY

request='{
  "event": {
    "event_id": "norm-smoke-earnings",
    "version": 1,
    "causal_parent_id": "raw-smoke-earnings",
    "event_type": "EARNINGS",
    "headline": "Quarterly earnings beat estimates",
    "body": "Profit rose and revenue grew higher than expected.",
    "occurred_at": "2026-07-06T09:15:00Z",
    "symbol": "RELIANCE",
    "sector": "Oil & Gas",
    "source": "NSE",
    "region": "IN",
    "impact_level": null,
    "impact_category": null
  },
  "facts": {
    "macro_context": {
      "sp500_futures_change": 0,
      "nasdaq_futures_change": 0,
      "brent_crude_change": 0,
      "usd_inr_change": 0,
      "fii_net_flow": 0,
      "gold_change": 0,
      "total_macro_score": 0
    },
    "entry_price": 1000,
    "exchange": "NSE",
    "features": null,
    "prediction": null,
    "kg_modifier": 0
  }
}'

decision="$(curl -fsS -X POST "$BASE_URL/decide" -H "Content-Type: application/json" -d "$request")"
python3 - "$decision" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["action"] == "BUY", payload
assert payload["execution_ready"] is True, payload
assert payload["quantity"] == 20, payload
assert payload["entry_price"] == 1000.0, payload
assert payload["target_price"] == 1030.0, payload
assert payload["stop_loss"] == 980.0, payload
assert payload["total_score"] == 0.72, payload
PY

echo "API smoke test passed against $BASE_URL"
