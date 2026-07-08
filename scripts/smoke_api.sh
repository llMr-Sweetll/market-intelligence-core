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

ready="$(curl -fsS "$BASE_URL/ready")"
python3 - "$ready" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["status"] == "ready", payload
assert payload["service"] == "gm-api", payload
assert "persistence" in payload, payload
PY

version="$(curl -fsS "$BASE_URL/version")"
python3 - "$version" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["service"] == "gm-api", payload
assert payload["version"], payload
assert payload["model_version"], payload
PY

openapi="$(curl -fsS "$BASE_URL/openapi.json")"
python3 - "$openapi" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["openapi"] == "3.1.0", payload
assert "/decide" in payload["paths"], payload
assert "/ready" in payload["paths"], payload
assert "/payments/state" in payload["paths"], payload
assert "/payments/orders" in payload["paths"], payload
assert "/payments/verify" in payload["paths"], payload
assert "/payments/webhooks/razorpay" in payload["paths"], payload
PY

payment_state="$(curl -fsS "$BASE_URL/payments/state")"
python3 - "$payment_state" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["mode"] == "TEST_MODE", payload
assert payload["live_billing_enabled"] is False, payload
assert payload["provider"]["name"] == "razorpay-test", payload
assert payload["provider"]["mode"] == "TEST_MODE", payload
PY

payment_order_request='{
  "account_id": "acct_release_smoke",
  "amount_paise": 49900,
  "currency": "INR",
  "description": "MV access",
  "success_url": "https://example.test/payments/success"
}'

payment_order="$(curl -fsS -X POST "$BASE_URL/payments/orders" -H "Content-Type: application/json" -d "$payment_order_request")"
python3 - "$payment_order" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["provider"] == "razorpay-test", payload
assert payload["key_id"], payload
assert payload["order_id"].startswith("order_test_"), payload
assert payload["test_payment_id"].startswith("pay_test_"), payload
assert payload["test_signature"], payload
PY

payment_verify_request="$(python3 - "$payment_order" <<'PY'
import json
import sys

order = json.loads(sys.argv[1])
print(json.dumps({
    "order_id": order["order_id"],
    "payment_id": order["test_payment_id"],
    "signature": order["test_signature"],
}))
PY
)"

payment_verification="$(curl -fsS -X POST "$BASE_URL/payments/verify" -H "Content-Type: application/json" -d "$payment_verify_request")"
python3 - "$payment_verification" "$payment_order" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
order = json.loads(sys.argv[2])
assert payload["verified"] is True, payload
assert payload["order_id"] == order["order_id"], payload
assert payload["payment_id"] == order["test_payment_id"], payload
PY

webhook_body='{"event":"payment.captured","payload":{"payment":{"entity":{"id":"pay_test_webhook","order_id":"order_test_webhook","amount":49900,"currency":"INR","captured":true}}}}'
webhook_signature="$(RAZORPAY_WEBHOOK_SECRET="${RAZORPAY_WEBHOOK_SECRET:-local_webhook_signing_key}" RAW_BODY="$webhook_body" python3 - <<'PY'
import hashlib
import hmac
import os

print(hmac.new(
    os.environ["RAZORPAY_WEBHOOK_SECRET"].encode(),
    os.environ["RAW_BODY"].encode(),
    hashlib.sha256,
).hexdigest())
PY
)"

webhook_verification="$(curl -fsS -X POST "$BASE_URL/payments/webhooks/razorpay" -H "Content-Type: application/json" -H "x-razorpay-signature: $webhook_signature" -d "$webhook_body")"
python3 - "$webhook_verification" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
assert payload["verification"]["verified"] is True, payload
assert payload["verification"]["event"] == "payment.captured", payload
assert payload["event"]["provider_order_id"] == "order_test_webhook", payload
assert payload["event"]["provider_payment_id"] == "pay_test_webhook", payload
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
assert payload["model_version"], payload
assert payload["input_hash"], payload
assert payload["expected_return"] is not None, payload
assert payload["downside"] is not None, payload
assert payload["explanation"]["pipeline"], payload
PY

echo "API smoke test passed against $BASE_URL"
