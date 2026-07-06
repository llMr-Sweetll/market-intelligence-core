# Payments

Razorpay support starts in test mode for `v0.1.0`.

## Goals

- create test-mode orders or subscriptions
- verify checkout signatures server-side
- verify webhooks using the raw request body
- store payment event history
- expose safe payment state in the UI

## Non-Goals

- live billing
- payouts
- refunds
- entitlement enforcement for real customers
- storing card, bank, or wallet details

## Environment

Expected variables:

```text
RAZORPAY_KEY_ID=
RAZORPAY_KEY_SECRET=
RAZORPAY_WEBHOOK_SECRET=
RAZORPAY_MODE=test
```

Only `RAZORPAY_KEY_ID` may be sent to the browser when needed by checkout.
Secrets stay on the server.

## Flow

```text
frontend requests checkout intent
  -> API creates test order/subscription
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

The payment PR must include:

- signature verification tests
- webhook fixture tests
- redaction tests for logs
- UI test-mode state
- failed-signature path

