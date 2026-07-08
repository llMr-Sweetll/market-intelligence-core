# API Contract

The HTTP API is a thin runtime wrapper over the deterministic domain core. The
decision path accepts explicit input facts and does not fetch data on its own.

## Operational Endpoints

- `GET /health` returns a liveness response for the API process.
- `GET /ready` reports service readiness and optional PostgreSQL status.
- `GET /version` returns the API version and current model version.
- `GET /openapi.json` returns the OpenAPI 3.1 contract.
- `GET /events` returns fixture-backed normalized event review summaries.
- `GET /events/{event_id}` returns raw metadata, normalized facts, entity
  mappings, and source reliability for one fixture event.
- `GET /payments/state` returns Razorpay test-mode state and recent verified
  payment events.
- `POST /payments/orders` creates a deterministic test-mode order.
- `POST /payments/verify` verifies a checkout signature.
- `POST /payments/webhooks/razorpay` verifies a signed raw webhook body.

When `DATABASE_URL` is unset, `/ready` still returns ready because persistence is
optional for local domain and smoke-test workflows. When `DATABASE_URL` is set,
the API connects to PostgreSQL at startup, applies migrations by default, and
`/ready` verifies that the database remains reachable.

## Decision Endpoint

`POST /decide` accepts:

- `event`: a normalized event with a stable `event_id` and `version`.
- `facts`: optional as-of facts such as macro context, entry price, exchange,
  features, prediction, and knowledge-graph modifier.

If persistence is configured, a successful decision writes:

- the normalized event
- the score projection on the event row
- matched rule traces
- the decision
- the replay input snapshot and input hash

If any configured persistence write fails, the endpoint returns `500` instead of
returning an unaudited decision.

Every decision response includes:

- `model_version`
- `input_hash`
- `expected_return`
- `downside`
- structured `explanation` with pipeline, evidence, gates, utilities, and
  missing facts

## Event Review Endpoints

`GET /events` is the release fixture feed for the Event Inbox. Each summary
includes:

- event ID and version
- headline and occurrence time
- source, region, sector, symbol, and event class
- confidence and severity
- entity mapping status
- source reliability score and tier

`GET /events/{event_id}` returns the selected review context:

- normalized event payload
- raw source metadata
- normalized facts
- entity mappings with entity type and confidence
- source reliability rationale

These endpoints are fixture-backed until live ingestion is enabled.

## Payment Endpoints

The payment endpoints are test-mode only. They do not enable live billing,
refunds, payouts, subscriptions, or entitlement enforcement.

`POST /payments/orders` accepts:

- `account_id`
- `amount_paise`
- `currency`
- `description`
- `success_url`

The response includes a public key ID, test order ID, test payment ID, and test
signature fixture for local smoke workflows. `POST /payments/verify` validates
the checkout signature using HMAC-SHA256 over `order_id|payment_id`.

`POST /payments/webhooks/razorpay` requires `X-Razorpay-Signature` and validates
the raw body with HMAC-SHA256 before accepting the event. If persistence is
configured, order and verification events are stored in PostgreSQL.

## Local Contract Check

```bash
make run-api
make smoke-api
```

The smoke script checks `/health`, `/ready`, `/version`, `/openapi.json`, the
payment test-mode order and signature flow, signed webhook verification, and the
deterministic BUY fixture on `/decide`. API unit tests also cover `/events`,
`/events/{event_id}`, and payment bad-signature paths.
