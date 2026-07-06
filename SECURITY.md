# Security

## Supported Versions

Security fixes target the current `main` branch until versioned releases are
published. After `v0.1.0`, supported release lines will be listed here.

## Reporting

Do not open public issues for secrets, payment verification failures, broker
credential handling problems, authentication bypasses, or data exposure.

Send a private report to the repository owner with:

- affected area
- reproduction steps
- expected impact
- logs or screenshots with secrets removed
- suggested fix, if known

## Secrets

Never commit:

- broker API keys or access tokens
- Razorpay key secrets or webhook secrets
- market-data provider tokens
- account identifiers tied to live financial accounts
- database passwords for shared environments

Use `.env` locally and repository secrets in automation.

## Trading Safety

`v0.1.0` must not place live broker orders. Broker integrations are limited to
read-only state, fixtures, and paper simulation until live execution is designed,
reviewed, and explicitly enabled in a later release.

## Payment Safety

Razorpay integration starts in test mode. Checkout signatures and webhooks must
be verified server-side. The frontend may receive public key identifiers but
must never receive key secrets or webhook secrets.

