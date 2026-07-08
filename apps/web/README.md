# Market Intelligence Core Web

The web app is the operator console for the Rust API. It checks `GET /health`,
posts the smoke-test event to `POST /decide`, and shows release-critical work
areas for events, decisions, knowledge, integrations, and payments.

The Decision Workbench renders the live decision response with action,
confidence, expected return, downside, model version, replay hash, risk gates,
evidence rows, utility estimates, input context, similar-event history, missing
facts, and replay path.

The Event Inbox reads fixture-backed normalized event summaries from `/events`,
filters by region, market, sector, event class, source, and severity, and loads
selected raw metadata, normalized facts, entity mappings, and source reliability
from `/events/{event_id}`.

## Local Development

Start the backend from the repository root:

```bash
make run-api
```

Start the web app:

```bash
npm install
npm run dev -- --host 127.0.0.1 --port 5173
```

The frontend defaults to `http://127.0.0.1:8000`. Override it when needed:

```bash
VITE_API_BASE_URL=http://127.0.0.1:9000 npm run dev
```

## Checks

```bash
npm run check
```

This runs linting, component tests, TypeScript compilation, and the production
build.

Run the browser smoke against the local Rust API and Vite app:

```bash
npm run test:e2e
```

The Playwright config starts `gm-api` without persistence and points Vite at the
temporary API port. The suite covers desktop and mobile layouts, Event Inbox,
Decision Workbench, chart rendering, accessibility, and UI decision-flow p95.
