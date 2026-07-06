# Knowledge Base

The knowledge base turns raw events into structured market context. It should
store facts, relationships, provenance, and event classifications without making
trading decisions by itself.

## Core Entities

- company
- instrument
- exchange
- country
- region
- regulator
- policy body
- sector
- industry
- commodity
- currency
- index
- broker
- source
- disease or medical classification
- conflict actor
- political organization

## Event Classes

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

- ICD-11 classification references
- public-health alerts
- drug approvals or warnings
- hospital, insurance, pharma, and logistics exposure
- disease outbreak classification

The system must not provide medical advice. Medical classifications are used
only to categorize events that may affect markets.

### Market Events

- price shock
- volume shock
- volatility regime shift
- liquidity drop
- currency movement
- commodity movement
- index rebalancing
- credit spread movement

## Relationships

Relationship examples:

- company issues instrument
- company belongs to sector
- regulator governs market
- policy affects sector
- conflict affects commodity
- commodity affects company margin
- medical classification affects public-health exposure
- source reported event
- event corroborates event

Each relationship should include:

- source
- confidence
- effective date
- optional expiry date
- provenance URL or source ID
- ingestion timestamp

## Provenance

Every fact needs enough provenance to answer:

- where did this come from?
- when was it observed?
- what entity was resolved?
- what confidence was assigned?
- what changed since the previous version?

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

