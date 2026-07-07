# UI/UX Plan

Market Intelligence Core should feel like an operator console for research,
market monitoring, and decision review. It should not feel like a marketing
site or a consumer trading game.

## Product Principles

- Evidence comes before action.
- State is explicit: healthy, stale, degraded, blocked, ready, simulated.
- Dense layouts are acceptable when they are readable and scannable.
- Every decision screen must show inputs, model version, assumptions, missing
  facts, and replay identifiers.
- Live broker order placement is not available in `v0.1.0`.
- Payments start in Razorpay test mode only.

## Recommended Stack

- Vite, React, and TypeScript for the web app.
- Tailwind CSS for layout and styling primitives.
- Accessible component primitives for dialogs, menus, tabs, tables, forms, and
  tooltips.
- TanStack Query for server-state caching, retries, invalidation, stale state,
  and request errors.
- Lightweight financial charts for price series, event markers, and abnormal
  return views.
- Vitest for component tests.
- Playwright for browser smoke tests and responsive checks.

## Layout

The first release should use a three-zone shell:

- Left navigation: Command Center, Event Inbox, Decision Workbench, Knowledge
  Graph, Market Impact, Integrations, Payments, Replay.
- Top status bar: API state, readiness state, provider health, model version,
  current environment, and last refresh.
- Main workspace: screen-specific tables, forms, evidence panels, charts, and
  audit trails.

Avoid nested cards. Use full-width workspace bands and only use cards for
repeated entities, modal content, or compact metric blocks.

## Visual System

Base tone:

- neutral background
- high-contrast text
- restrained accent colors
- compact tables
- clear borders and dividers

Semantic colors:

- green: healthy, verified, execution-ready simulation
- amber: stale, missing facts, watch state
- red: failed, blocked, unsafe, live execution disabled
- blue: informational state
- violet: model/version metadata only

Buttons:

- icon buttons for common actions
- text buttons for commands with business meaning
- disabled states must explain why through tooltip or inline context

## Screens

### Command Center

Purpose: show whether the system is alive, current, and safe to use.

Content:

- API health and readiness
- model version
- provider health
- ingestion status
- latest events
- latest decisions
- degraded-state warnings

### Event Inbox

Purpose: review normalized events before they become decision inputs.

Content:

- event table with filters for region, sector, class, source, severity, and
  confidence
- expandable raw source metadata
- normalized event facts
- entity mapping status
- source reliability
- duplicate and related-event hints

Current release surface:

- loads fixture-backed normalized event summaries from `/events`
- filters by region, market, sector, event class, source, and severity
- selects an event and loads `/events/{event_id}`
- displays raw source metadata, normalized facts, entity mappings, source
  reliability, severity, confidence, and mapping status
- supports manual fixture review before live ingestion is enabled

### Decision Workbench

Purpose: evaluate one event with explicit as-of facts.

Content:

- event payload editor or guided form
- facts panel for price, macro, features, prediction, and relationship context
- evidence panel for historical similar events, event-study metrics, and risk
  gates
- decision result with action, confidence, expected utility, missing facts, and
  explanation
- replay ID and model version

Current release surface:

- submits the fixture event and as-of facts to the local API
- displays action, confidence, expected return, downside, quantity, target, and
  stop
- displays model version, input hash, parent event, and event version for replay
- displays risk gates, rule/event-study evidence, action utilities, input
  context, similar-event history, missing facts, and replay pipeline
- keeps live broker execution unavailable; executable output means local
  decision readiness only

### Knowledge Graph

Purpose: inspect relationships that influence event interpretation.

Content:

- company, instrument, country, regulator, sector, commodity, disease or
  classification, source, and conflict actor nodes
- relationship list with provenance
- event-class filters
- impact-path view from event to affected markets

### Market Impact

Purpose: show price behavior around events.

Content:

- price chart with event markers
- benchmark-relative return
- abnormal return
- cumulative abnormal return window
- volatility and liquidity context
- similar-event comparison table

### Integrations

Purpose: configure and observe providers without exposing secrets.

Content:

- provider cards for market data, events, filings, entity mapping, brokers, and
  payments
- configured/test/degraded/rate-limited states
- last successful request timestamp
- redacted credential status
- test connection action

### Payments

Purpose: manage Razorpay test-mode payment state.

Content:

- test-mode order and subscription state
- webhook event log
- signature verification status
- failure reasons
- no secret display

### Replay

Purpose: prove determinism.

Content:

- stored decision inputs
- model version
- replay result
- old-vs-new comparison when model versions differ
- deterministic decision ID

## Required States

Every API-backed screen needs:

- loading state
- empty state
- success state
- validation error
- server error
- stale data warning
- provider rate-limit warning
- offline or backend-unreachable state

## Verification

The UI is release-ready when:

- the app shell renders on desktop and mobile
- primary text does not overlap or clip
- Decision Workbench completes against the local backend
- chart surfaces are nonblank with fixture data
- keyboard navigation reaches primary controls
- Playwright covers Command Center and Decision Workbench against the local API
- screenshots are attached to the frontend PR
