# Knowledge Base

The knowledge base turns raw events into structured market context. It should
store facts, relationships, provenance, and event classifications without making
trading decisions by itself.

## Core Entities

The Rust domain crate defines the release entity types in
`gm_domain::EntityType`:

- company
- instrument
- country
- regulator
- disease classification
- commodity
- sector
- index
- broker
- source
- exchange
- region
- policy body
- conflict actor
- political organization
- currency

## Event Classes

The Rust domain crate defines the release event classes in
`gm_domain::EventClass`. The first release supports:

### Company Events

- earnings
- guidance
- buyback
- dividend
- split
- merger or acquisition
- management change
- board change
- debt change
- shareholding change
- subsidiary or restructuring change
- regulatory filing

### Regulation and Policy

- regulation
- policy change
- rate decision
- tax policy
- trade restriction
- sanctions
- market regulation
- sector rule change
- exchange circular
- public-health policy
- environmental policy

### Global Movements

- global movement
- political meeting
- conflict
- protests
- strikes
- migration pressure
- supply-chain disruption
- diplomatic meetings
- election-related events
- alliance changes
- military escalation
- ceasefire or peace talks

### Medical and Health Classification

- medical classification
- ICD-11 classification references
- public-health alerts
- drug approvals or warnings
- hospital, insurance, pharma, and logistics exposure
- disease outbreak classification

The system must not provide medical advice. Medical classifications are used
only to categorize events that may affect markets.

### Market Events

- macro event
- market event
- price shock
- volume shock
- volatility regime shift
- liquidity drop
- currency movement
- commodity movement
- index rebalancing
- credit spread movement

## Relationships

The Rust domain crate defines relationship types in
`gm_domain::RelationshipType`:

- company issues instrument
- company belongs to sector
- instrument listed on index
- regulator governs market or index
- policy affects sector
- conflict affects commodity
- commodity affects company margin
- medical classification affects sector
- source reports event
- broker provides paper execution
- country hosts company
- event corroborates event

Each relationship should include:

- source
- confidence
- effective date
- optional expiry date
- provenance URL or source ID
- ingestion timestamp

The release fixture graph is exposed through `gm_domain::mv_fixture_graph()`.
It includes representative company, instrument, country, regulator, policy,
medical classification, commodity, sector, index, broker, source, and conflict
actor nodes. Relationships include market structure, policy-to-sector,
conflict-to-commodity, commodity-to-margin, medical-classification-to-sector,
source-reporting, paper-broker, and corroboration paths.

## Provenance

Every fact needs enough provenance to answer:

- where did this come from?
- when was it observed?
- what entity was resolved?
- what confidence was assigned?
- what changed since the previous version?

The implemented `Provenance` shape is:

- `source`
- `source_id`
- `url`
- `observed_at`
- `confidence`

Tests enforce that fixture relationships carry non-empty identifiers,
confidence in `(0, 1]`, an effective date, and a provenance URL.

## Release Data Sources

Candidate sources for `v0.1.0` and follow-up releases:

- GDELT for global news and event categories
- ACLED for conflict, protest, and political violence data
- Marketaux or Alpha Vantage for market news and entity-linked articles
- SEC EDGAR for public company filings
- OpenFIGI for instrument identifier mapping
- World Bank for macro indicators
- WHO ICD API for medical classification references
- NSE/BSE public pages or approved providers for Indian corporate actions

Live provider calls should enter through integration adapters, not domain math.
