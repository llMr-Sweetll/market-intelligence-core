# Integrations

Integrations collect facts. They do not decide actions directly.

## Adapter Traits

The `gm-integrations` crate owns the adapter contracts and deterministic
fixtures. The domain crate does not depend on providers.

The integration layer should expose these boundaries:

- `MarketDataProvider`
- `EventProvider`
- `FilingProvider`
- `EntityMappingProvider`
- `PaymentProvider`
- `ExecutionProvider`

Each adapter should report:

- provider name
- mode: fixture, test, sandbox, live-read-only
- health
- last success
- last error
- rate-limit state
- retry state
- circuit-breaker state
- credential status without exposing secrets

## First Providers

### Fixture Provider

Required for all release tests. It should provide deterministic events, prices,
filings, and payment webhook samples.

Implemented fixtures cover market data, normalized events, filings, entity
mapping, Razorpay-style test-mode checkout/webhook flow, and paper execution.

### GDELT

Use for global movements, diplomatic events, conflict-related news, and broad
cross-market signals.

### Marketaux or Alpha Vantage

Use for entity-linked market news and financial sentiment where the data license
and rate limits fit the release.

### SEC EDGAR

Use for public company filings and company-structure events in the US market.

### OpenFIGI

Use for mapping external identifiers to stable instrument metadata.

### Broker Adapters

Zerodha, Upstox, and Dhan belong behind read-only or paper interfaces for
`v0.1.0`.

Live order placement is disabled in the first release. Any future live path must
include:

- explicit configuration
- static IP and broker requirements
- rate-limit enforcement
- risk checks
- audit log
- manual enablement
- tests proving disabled-by-default behavior

The current paper execution provider rejects live orders by construction.

## Error Handling

Adapters must classify errors:

- authentication
- rate limit
- timeout
- malformed response
- provider unavailable
- stale data
- unsupported operation

Provider errors should not crash the domain path. The API should return degraded
provider state and continue to serve deterministic fixture workflows.

## Secrets

Secrets live in environment variables or hosted secret stores only. Frontend code
must never receive provider secrets.
