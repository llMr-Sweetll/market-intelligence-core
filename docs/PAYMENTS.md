# Payments

Razorpay support is implemented in test mode for `v0.1.0`.

## Goals

- create test-mode orders
- verify checkout signatures server-side
- verify webhooks using the raw request body
- store payment event history
- expose safe payment state in the UI

## Non-Goals

- live billing
- payouts
- refunds
- subscriptions
- entitlement enforcement for real customers
- storing card, bank, or wallet details

## Environment

Expected variables:

```text
RAZORPAY_KEY_ID=
RAZORPAY_KEY_SECRET=
RAZORPAY_WEBHOOK_SECRET=
```

Only `RAZORPAY_KEY_ID` may be sent to the browser when needed by checkout.
Secrets stay on the server.

The local defaults are deterministic test fixtures. Production values must be
provided through the runtime environment, not committed files.

## API Endpoints

- `GET /payments/state` returns provider health, test-mode status,
  verification method names, and recent persisted payment events.
- `POST /payments/orders` creates a deterministic test order using subunit
  amounts.
- `POST /payments/verify` verifies the checkout return signature using
  HMAC-SHA256 over `order_id|payment_id`.
- `POST /payments/webhooks/razorpay` verifies `X-Razorpay-Signature` using
  HMAC-SHA256 over the raw request body.

## Flow

```text
frontend requests checkout intent
  -> API creates test order
  -> frontend opens checkout with public key ID
  -> frontend returns payment identifiers
  -> API verifies checkout signature
  -> Razorpay sends webhook
  -> API verifies webhook signature from raw body
  -> API stores payment event
  -> UI shows verified payment state
```

## Webhook Events

Initial events:

- `payment.captured`
- `payment.failed`
- `order.paid`
- subscription lifecycle events when subscriptions are enabled

## Verification

The implemented test-mode path is covered by:

- signature verification tests
- webhook fixture tests
- local API smoke test coverage
- UI test-mode state
- failed-signature path
- Playwright coverage for the visible test payment flow
