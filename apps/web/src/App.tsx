import { useEffect, useMemo, useRef, useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import {
  Activity,
  AlertTriangle,
  ArrowUpRight,
  Banknote,
  Bell,
  Building2,
  CheckCircle2,
  CircleDollarSign,
  ClipboardCheck,
  Database,
  Fingerprint,
  GitBranch,
  Globe2,
  Landmark,
  LineChart,
  ListChecks,
  Network,
  PlugZap,
  RefreshCw,
  Scale,
  ShieldCheck,
  Stethoscope,
  TrendingUp,
} from 'lucide-react'
import { createChart, LineSeries, type LineData } from 'lightweight-charts'

import {
  createDecision,
  createPaymentOrder,
  fetchEvent,
  fetchEvents,
  fetchHealth,
  fetchPaymentState,
  getApiBaseUrl,
  verifyPayment,
  type Decision,
  type DecisionRequest,
  type EventReviewDetail,
  type EventReviewSummary,
  type HealthResponse,
  type PaymentOrder,
  type PaymentState,
  type PaymentVerification,
} from './api'
import { earningsDecisionRequest, impactSeries } from './fixtures'

type StatusTone = 'green' | 'amber' | 'red' | 'blue' | 'neutral'

type Metric = {
  label: string
  value: string
  detail: string
  tone: StatusTone
}

type EventFilterState = {
  region: string
  market: string
  sector: string
  eventClass: string
  source: string
  severity: string
}

type EventFilterOptions = Record<keyof EventFilterState, string[]>

type KnowledgeRow = {
  label: string
  scope: string
  owner: string
  icon: typeof Globe2
}

type IntegrationRow = {
  label: string
  status: string
  mode: string
  detail: string
  tone: StatusTone
}

type PaymentFlowResult = {
  order: PaymentOrder
  verification: PaymentVerification
}

const metrics: Metric[] = [
  { label: 'API status', value: 'Live check', detail: 'GET /health', tone: 'blue' },
  { label: 'Decision path', value: 'Deterministic', detail: 'No clock or random input', tone: 'green' },
  { label: 'Order mode', value: 'Disabled', detail: 'Read-only plus paper trading', tone: 'amber' },
  { label: 'Payments', value: 'Test mode', detail: 'Signed checkout and webhook verification', tone: 'blue' },
]

const knowledgeRows: KnowledgeRow[] = [
  { label: 'Global movements', scope: 'Markets, flows, blocs', owner: 'macro graph', icon: Globe2 },
  { label: 'Political reunions', scope: 'Alliances and summits', owner: 'event graph', icon: Landmark },
  { label: 'Medical classifications', scope: 'Therapy, codes, reimbursement', owner: 'sector graph', icon: Stethoscope },
  { label: 'Company structure', scope: 'Ownership, policy, filings', owner: 'entity graph', icon: Building2 },
]

const integrationRows: IntegrationRow[] = [
  {
    label: 'Zerodha',
    status: 'Paper trading',
    mode: 'Broker adapter',
    detail: 'Read-only positions first; live orders remain blocked.',
    tone: 'amber',
  },
  {
    label: 'Razorpay',
    status: 'Test mode',
    mode: 'Payments',
    detail: 'Checkout signatures and raw-body webhooks verified in test mode.',
    tone: 'blue',
  },
  {
    label: 'Global event feeds',
    status: 'Adapter queue',
    mode: 'Knowledge ingest',
    detail: 'Policy, conflict, market, and company events share one normalized envelope.',
    tone: 'neutral',
  },
  {
    label: 'Search and money data',
    status: 'Mock provider',
    mode: 'Market facts',
    detail: 'Contract tests lock shape before paid or rate-limited providers are added.',
    tone: 'green',
  },
]

function App() {
  const [eventFilters, setEventFilters] = useState({
    region: 'all',
    market: 'all',
    sector: 'all',
    eventClass: 'all',
    source: 'all',
    severity: 'all',
  })
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null)

  const health = useQuery<HealthResponse>({
    queryKey: ['health'],
    queryFn: fetchHealth,
    retry: false,
    refetchInterval: 30_000,
  })

  const eventInbox = useQuery<EventReviewSummary[]>({
    queryKey: ['events'],
    queryFn: fetchEvents,
    retry: false,
  })

  const paymentState = useQuery<PaymentState>({
    queryKey: ['payments', 'state'],
    queryFn: fetchPaymentState,
    retry: false,
  })

  const eventDetail = useQuery<EventReviewDetail>({
    queryKey: ['event', selectedEventId],
    queryFn: () => fetchEvent(selectedEventId ?? ''),
    enabled: selectedEventId !== null,
    retry: false,
  })

  const decision = useMutation<Decision, Error, void>({
    mutationFn: () => createDecision(earningsDecisionRequest),
  })

  const paymentFlow = useMutation<PaymentFlowResult, Error, void>({
    mutationFn: async () => {
      const order = await createPaymentOrder({
        account_id: 'acct_operator_mv',
        amount_paise: 49900,
        currency: 'INR',
        description: 'MV access',
        success_url: `${window.location.origin}/payments/success`,
      })
      const verification = await verifyPayment({
        order_id: order.order_id,
        payment_id: order.test_payment_id,
        signature: order.test_signature,
      })

      return { order, verification }
    },
    onSuccess: () => {
      void paymentState.refetch()
    },
  })

  const apiState = useMemo(() => {
    if (health.isPending) {
      return { label: 'Checking', tone: 'neutral' as StatusTone }
    }

    if (health.isSuccess && health.data.status === 'ok') {
      return { label: 'Online', tone: 'green' as StatusTone }
    }

    return { label: 'Offline', tone: 'red' as StatusTone }
  }, [health.data?.status, health.isPending, health.isSuccess])

  const filterOptions = useMemo(
    () => buildEventFilterOptions(eventInbox.data ?? []),
    [eventInbox.data],
  )

  const visibleEvents = useMemo(
    () => filterEvents(eventInbox.data ?? [], eventFilters),
    [eventFilters, eventInbox.data],
  )

  useEffect(() => {
    if (!eventInbox.data?.length) {
      return
    }

    setSelectedEventId((current) => current ?? eventInbox.data[0]?.event_id ?? null)
  }, [eventInbox.data])

  useEffect(() => {
    if (visibleEvents.length === 0) {
      return
    }

    if (!visibleEvents.some((event) => event.event_id === selectedEventId)) {
      setSelectedEventId(visibleEvents[0]?.event_id ?? null)
    }
  }, [selectedEventId, visibleEvents])

  return (
    <div className="app-shell">
      <aside className="sidebar" aria-label="Primary">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">
            MI
          </div>
          <div>
            <p className="eyebrow">Market Intelligence</p>
            <h1>Command Center</h1>
          </div>
        </div>

        <nav className="nav-list" aria-label="Workspace">
          <a href="#command">
            <Activity size={18} aria-hidden="true" />
            Command
          </a>
          <a href="#events">
            <Bell size={18} aria-hidden="true" />
            Events
          </a>
          <a href="#decision">
            <GitBranch size={18} aria-hidden="true" />
            Decision
          </a>
          <a href="#knowledge">
            <Network size={18} aria-hidden="true" />
            Knowledge
          </a>
          <a href="#integrations">
            <PlugZap size={18} aria-hidden="true" />
            Integrations
          </a>
        </nav>

        <div className={`status-box tone-${apiState.tone}`}>
          <span className="status-dot" aria-hidden="true" />
          <div>
            <strong>{apiState.label}</strong>
            <span>{getApiBaseUrl()}</span>
          </div>
        </div>
      </aside>

      <main className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">MV release console</p>
            <h2>Deterministic market decision operations</h2>
          </div>
          <button className="icon-button" type="button" onClick={() => void health.refetch()}>
            <RefreshCw size={18} aria-hidden="true" />
            <span>Refresh API</span>
          </button>
        </header>

        <section id="command" className="panel command-panel" aria-labelledby="command-heading">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Command Center</p>
              <h3 id="command-heading">Release readiness</h3>
            </div>
            <span className="release-pill">v0.1.0 MV</span>
          </div>

          <div className="metric-grid">
            {metrics.map((metric) => (
              <div className={`metric-cell tone-${metric.tone}`} key={metric.label}>
                <span>{metric.label}</span>
                <strong>{metric.value}</strong>
                <small>{metric.detail}</small>
              </div>
            ))}
          </div>
        </section>

        <section id="events" className="panel" aria-labelledby="events-heading">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Event Inbox</p>
              <h3 id="events-heading">Normalized review queue</h3>
            </div>
            <button
              className="ghost-button"
              type="button"
              onClick={() => void eventInbox.refetch()}
              disabled={eventInbox.isFetching}
            >
              <Database size={17} aria-hidden="true" />
              {eventInbox.isFetching ? 'Loading fixtures' : 'Reload fixtures'}
            </button>
          </div>

          <EventFilters
            filters={eventFilters}
            options={filterOptions}
            onChange={setEventFilters}
          />

          {eventInbox.error ? (
            <div className="event-state error-state" role="alert">
              <p className="eyebrow">Events</p>
              <h4>Event API unavailable</h4>
              <p>{eventInbox.error.message}</p>
            </div>
          ) : eventInbox.isPending ? (
            <div className="event-state empty-state">
              <p className="eyebrow">Events</p>
              <h4>Loading fixture events</h4>
              <p>Fetching normalized review data from /events.</p>
            </div>
          ) : (
            <div className="event-inbox-layout">
              <div className="event-table" role="table" aria-label="Normalized events">
                <div className="event-table-head" role="row">
                  <span role="columnheader">Event</span>
                  <span role="columnheader">Source</span>
                  <span role="columnheader">Class</span>
                  <span role="columnheader">Symbol</span>
                  <span role="columnheader">Severity</span>
                </div>
                {visibleEvents.length > 0 ? (
                  visibleEvents.map((event) => (
                    <div
                      className={`event-row ${selectedEventId === event.event_id ? 'is-selected' : ''}`}
                      role="row"
                      key={event.event_id}
                    >
                      <button
                        className="event-title-button"
                        type="button"
                        role="cell"
                        onClick={() => setSelectedEventId(event.event_id)}
                      >
                        <strong>{event.headline}</strong>
                        <small>{formatDateTime(event.occurred_at)}</small>
                      </button>
                      <span role="cell">{event.source ?? '-'}</span>
                      <span role="cell">{formatLabel(event.event_class)}</span>
                      <span role="cell">{event.symbol ?? '-'}</span>
                      <span className="priority" role="cell">
                        {event.severity}
                      </span>
                    </div>
                  ))
                ) : (
                  <div className="event-row empty-event-row" role="row">
                    <span role="cell">No events match the selected filters.</span>
                  </div>
                )}
              </div>

              <EventDetailPanel
                detail={eventDetail.data}
                error={eventDetail.error}
                isLoading={eventDetail.isPending || eventDetail.isFetching}
              />
            </div>
          )}
        </section>

        <section
          id="decision"
          className="panel decision-panel"
          aria-labelledby="decision-heading"
        >
          <div className="section-heading">
            <div>
              <p className="eyebrow">Decision Workbench</p>
              <h3 id="decision-heading">Evidence and audit trail</h3>
            </div>
            <button
              className="primary-button"
              type="button"
              onClick={() => decision.mutate()}
              disabled={decision.isPending}
            >
              <TrendingUp size={17} aria-hidden="true" />
              {decision.isPending ? 'Running' : 'Run fixture'}
            </button>
          </div>

          <div className="decision-layout">
            <div className="request-summary">
              <p className="eyebrow">Input</p>
              <h4>{earningsDecisionRequest.event.headline}</h4>
              <dl>
                <div>
                  <dt>Symbol</dt>
                  <dd>{earningsDecisionRequest.event.symbol}</dd>
                </div>
                <div>
                  <dt>Entry</dt>
                  <dd>{formatCurrency(earningsDecisionRequest.facts.entry_price)}</dd>
                </div>
                <div>
                  <dt>Sector</dt>
                  <dd>{earningsDecisionRequest.event.sector}</dd>
                </div>
                <div>
                  <dt>Exchange</dt>
                  <dd>{earningsDecisionRequest.facts.exchange}</dd>
                </div>
                <div>
                  <dt>Event ID</dt>
                  <dd className="mono">{earningsDecisionRequest.event.event_id}</dd>
                </div>
                <div>
                  <dt>Evidence</dt>
                  <dd>{earningsDecisionRequest.facts.event_study?.abnormal_returns.length ?? 0} samples</dd>
                </div>
              </dl>
            </div>

            <DecisionResult
              decision={decision.data}
              error={decision.error}
              request={earningsDecisionRequest}
            />
          </div>
        </section>

        <section id="knowledge" className="panel" aria-labelledby="knowledge-heading">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Knowledge Base</p>
              <h3 id="knowledge-heading">Entity and event expansion</h3>
            </div>
            <span className="subtle-status">Ontology first</span>
          </div>

          <div className="knowledge-grid">
            {knowledgeRows.map((item) => {
              const Icon = item.icon
              return (
                <div className="knowledge-row" key={item.label}>
                  <Icon size={20} aria-hidden="true" />
                  <div>
                    <strong>{item.label}</strong>
                    <span>{item.scope}</span>
                  </div>
                  <small>{item.owner}</small>
                </div>
              )
            })}
          </div>
        </section>

        <section className="panel chart-panel" aria-labelledby="market-heading">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Market Impact</p>
              <h3 id="market-heading">Event-study response curve</h3>
            </div>
            <LineChart size={22} aria-hidden="true" />
          </div>
          <ImpactChart />
        </section>

        <section id="integrations" className="panel" aria-labelledby="integrations-heading">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Integrations</p>
              <h3 id="integrations-heading">Provider adapter map</h3>
            </div>
            <ShieldCheck size={22} aria-hidden="true" />
          </div>

          <div className="integration-list">
            {integrationRows.map((integration) => (
              <div className="integration-row" key={integration.label}>
                <div>
                  <strong>{integration.label}</strong>
                  <span>{integration.mode}</span>
                </div>
                <p>{integration.detail}</p>
                <span className={`status-chip tone-${integration.tone}`}>{integration.status}</span>
              </div>
            ))}
          </div>
        </section>

        <PaymentPanel
          state={paymentState.data}
          error={paymentState.error}
          flowError={paymentFlow.error}
          flowResult={paymentFlow.data}
          isLoading={paymentState.isPending || paymentState.isFetching}
          isRunning={paymentFlow.isPending}
          onRun={() => paymentFlow.mutate()}
        />
      </main>
    </div>
  )
}

function PaymentPanel({
  state,
  error,
  flowError,
  flowResult,
  isLoading,
  isRunning,
  onRun,
}: {
  state: PaymentState | undefined
  error: Error | null
  flowError: Error | null
  flowResult: PaymentFlowResult | undefined
  isLoading: boolean
  isRunning: boolean
  onRun: () => void
}) {
  const providerMode = state ? formatLabel(state.provider.mode) : 'Unknown'
  const providerHealth = state ? formatLabel(state.provider.health) : 'Unknown'
  const recentEventCount = state?.recent_events.length ?? 0
  const flowStatus = flowResult?.verification.verified ? 'Verified' : 'Waiting'
  const orderLabel = flowResult?.order.order_id ? shortHash(flowResult.order.order_id) : '-'
  const paymentLabel = flowResult?.verification.payment_id
    ? shortHash(flowResult.verification.payment_id)
    : '-'

  return (
    <section className="panel payment-panel" aria-labelledby="payments-heading">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Payments</p>
          <h3 id="payments-heading">Test-mode payment gate</h3>
        </div>
        <div className="payment-actions">
          <span className={`status-chip tone-${state?.live_billing_enabled ? 'red' : 'blue'}`}>
            {state?.live_billing_enabled ? 'Live billing' : 'Test mode'}
          </span>
          <button className="primary-button" type="button" onClick={onRun} disabled={isRunning}>
            <CircleDollarSign size={17} aria-hidden="true" />
            {isRunning ? 'Running' : 'Run test payment'}
          </button>
        </div>
      </div>

      {error ? (
        <div className="payment-state error-state" role="alert">
          <p className="eyebrow">Payments</p>
          <h4>Payment API unavailable</h4>
          <p>{error.message}</p>
        </div>
      ) : flowError ? (
        <div className="payment-state error-state" role="alert">
          <p className="eyebrow">Payments</p>
          <h4>Payment check failed</h4>
          <p>{flowError.message}</p>
        </div>
      ) : isLoading ? (
        <div className="payment-state empty-state">
          <p className="eyebrow">Payments</p>
          <h4>Loading payment state</h4>
          <p>Fetching provider mode and recent verification events.</p>
        </div>
      ) : null}

      <div className="payment-grid">
        <div>
          <Banknote size={20} aria-hidden="true" />
          <strong>{state?.provider.name ?? 'Razorpay test'}</strong>
          <span>
            {providerMode} · {providerHealth}
          </span>
        </div>
        <div>
          <CheckCircle2 size={20} aria-hidden="true" />
          <strong>Checkout signature</strong>
          <span>{state ? formatLabel(state.checkout_verification) : '-'}</span>
        </div>
        <div>
          <ShieldCheck size={20} aria-hidden="true" />
          <strong>Webhook signature</strong>
          <span>{state ? formatLabel(state.webhook_verification) : '-'}</span>
        </div>
        <div>
          <ArrowUpRight size={20} aria-hidden="true" />
          <strong>{flowStatus}</strong>
          <span>Order {orderLabel}</span>
        </div>
        <div>
          <Fingerprint size={20} aria-hidden="true" />
          <strong>Payment ID</strong>
          <span>{paymentLabel}</span>
        </div>
        <div>
          <Database size={20} aria-hidden="true" />
          <strong>{recentEventCount} webhook events</strong>
          <span>{state?.webhook_verification ? 'Raw body HMAC' : '-'}</span>
        </div>
      </div>
    </section>
  )
}

function EventFilters({
  filters,
  options,
  onChange,
}: {
  filters: EventFilterState
  options: EventFilterOptions
  onChange: (filters: EventFilterState) => void
}) {
  const setFilter = (key: keyof EventFilterState, value: string) => {
    onChange({ ...filters, [key]: value })
  }

  return (
    <div className="event-filter-grid" aria-label="Event filters">
      <FilterSelect
        label="Region"
        value={filters.region}
        options={options.region}
        onChange={(value) => setFilter('region', value)}
      />
      <FilterSelect
        label="Market"
        value={filters.market}
        options={options.market}
        onChange={(value) => setFilter('market', value)}
      />
      <FilterSelect
        label="Sector"
        value={filters.sector}
        options={options.sector}
        onChange={(value) => setFilter('sector', value)}
      />
      <FilterSelect
        label="Event class"
        value={filters.eventClass}
        options={options.eventClass}
        onChange={(value) => setFilter('eventClass', value)}
      />
      <FilterSelect
        label="Source"
        value={filters.source}
        options={options.source}
        onChange={(value) => setFilter('source', value)}
      />
      <FilterSelect
        label="Severity"
        value={filters.severity}
        options={options.severity}
        onChange={(value) => setFilter('severity', value)}
      />
    </div>
  )
}

function FilterSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string
  value: string
  options: string[]
  onChange: (value: string) => void
}) {
  return (
    <label className="filter-select">
      <span>{label}</span>
      <select value={value} onChange={(event) => onChange(event.target.value)}>
        <option value="all">All</option>
        {options.map((option) => (
          <option value={option} key={option}>
            {formatLabel(option)}
          </option>
        ))}
      </select>
    </label>
  )
}

function EventDetailPanel({
  detail,
  error,
  isLoading,
}: {
  detail: EventReviewDetail | undefined
  error: Error | null
  isLoading: boolean
}) {
  if (error) {
    return (
      <div className="event-detail error-state" role="alert">
        <p className="eyebrow">Event detail</p>
        <h4>Detail unavailable</h4>
        <p>{error.message}</p>
      </div>
    )
  }

  if (isLoading || !detail) {
    return (
      <div className="event-detail empty-state">
        <p className="eyebrow">Event detail</p>
        <h4>Loading selected event</h4>
        <p>Fetching raw metadata, normalized facts, and mappings.</p>
      </div>
    )
  }

  return (
    <div className="event-detail" aria-label="Selected event detail">
      <div className="event-detail-head">
        <div>
          <p className="eyebrow">Selected Event</p>
          <h4>{detail.summary.headline}</h4>
        </div>
        <span className="status-chip tone-blue">{formatPercent(detail.summary.confidence)}</span>
      </div>

      <dl className="event-detail-grid">
        <div>
          <dt>Class</dt>
          <dd>{formatLabel(detail.summary.event_class)}</dd>
        </div>
        <div>
          <dt>Reliability</dt>
          <dd>
            {formatLabel(detail.source_reliability.tier)} ·{' '}
            {formatPercent(detail.source_reliability.score)}
          </dd>
        </div>
        <div>
          <dt>Source ID</dt>
          <dd className="mono">{detail.raw_source.source_id}</dd>
        </div>
        <div>
          <dt>Mapping</dt>
          <dd>{formatLabel(detail.summary.entity_mapping_status)}</dd>
        </div>
      </dl>

      <div className="event-detail-section">
        <h5>Raw source metadata</h5>
        <p>{detail.raw_source.raw_headline}</p>
        <small>
          {detail.raw_source.provider} · {detail.raw_source.language.toUpperCase()} ·{' '}
          {formatDateTime(detail.raw_source.received_at)}
        </small>
      </div>

      <div className="event-detail-section">
        <h5>Normalized facts</h5>
        <dl className="context-list">
          {Object.entries(detail.normalized_facts).map(([key, value]) => (
            <div key={key}>
              <dt>{formatLabel(key)}</dt>
              <dd>{value ?? '-'}</dd>
            </div>
          ))}
        </dl>
      </div>

      <div className="event-detail-section">
        <h5>Entity mappings</h5>
        <div className="mapping-list">
          {detail.entity_mappings.map((mapping) => (
            <div className="mapping-row" key={mapping.entity_id}>
              <div>
                <strong>{mapping.label}</strong>
                <span>{formatLabel(mapping.entity_type)}</span>
              </div>
              <code>{mapping.entity_id}</code>
              <span>{formatPercent(mapping.confidence)}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="event-detail-section">
        <h5>Source reliability</h5>
        <p>{detail.source_reliability.rationale}</p>
      </div>
    </div>
  )
}

function DecisionResult({
  decision,
  error,
  request,
}: {
  decision: Decision | undefined
  error: Error | null
  request: DecisionRequest
}) {
  if (error) {
    return (
      <div className="decision-result error-state" role="alert">
        <p className="eyebrow">Result</p>
        <h4>Request failed</h4>
        <p>{error.message}</p>
      </div>
    )
  }

  if (!decision) {
    return (
      <div className="decision-result empty-state">
        <p className="eyebrow">Result</p>
        <h4>Waiting for backend response</h4>
        <p>Run the fixture to POST the smoke-test event to /decide.</p>
      </div>
    )
  }

  return (
    <div className="decision-result">
      <div className="result-head">
        <div>
          <p className="eyebrow">Result</p>
          <h4>{decision.action}</h4>
        </div>
        <span className={`action-badge action-${decision.action.toLowerCase()}`}>
          {decision.execution_ready ? 'Paper ready' : 'Review'}
        </span>
      </div>

      <div className="audit-strip" aria-label="Decision replay metadata">
        <div>
          <Fingerprint size={18} aria-hidden="true" />
          <span>Input hash</span>
          <strong className="mono" title={decision.input_hash}>
            {shortHash(decision.input_hash)}
          </strong>
        </div>
        <div>
          <GitBranch size={18} aria-hidden="true" />
          <span>Model</span>
          <strong>{decision.model_version}</strong>
        </div>
        <div>
          <ClipboardCheck size={18} aria-hidden="true" />
          <span>Replay</span>
          <strong>
            {decision.parent_event_id} v{decision.parent_event_version}
          </strong>
        </div>
      </div>

      <dl className="result-grid">
        <div>
          <dt>Score</dt>
          <dd>{decision.total_score.toFixed(2)}</dd>
        </div>
        <div>
          <dt>Confidence</dt>
          <dd>{formatPercent(decision.confidence)}</dd>
        </div>
        <div>
          <dt>Quantity</dt>
          <dd>{decision.quantity ?? '-'}</dd>
        </div>
        <div>
          <dt>Target</dt>
          <dd>{formatCurrency(decision.target_price)}</dd>
        </div>
        <div>
          <dt>Expected return</dt>
          <dd>{formatSignedPercent(decision.expected_return)}</dd>
        </div>
        <div>
          <dt>Downside</dt>
          <dd>{formatSignedPercent(decision.downside)}</dd>
        </div>
        <div>
          <dt>Stop</dt>
          <dd>{formatCurrency(decision.stop_loss)}</dd>
        </div>
        <div>
          <dt>ID</dt>
          <dd className="mono">{decision.decision_id.slice(0, 8)}</dd>
        </div>
      </dl>

      <p className="thesis">{decision.thesis}</p>

      <ModelReportView decision={decision} request={request} />
    </div>
  )
}

function ModelReportView({
  decision,
  request,
}: {
  decision: Decision
  request: DecisionRequest
}) {
  const explanation = decision.explanation
  const missingFacts = explanation.missing_facts
  const eventStudy = explanation.impact.event_study
  const featureState = request.facts.features ? 'Supplied' : 'Not supplied'
  const predictionState = request.facts.prediction ? 'Supplied' : 'Not supplied'

  return (
    <div className="model-report" aria-label="Model explanation">
      <section className="report-section" aria-labelledby="summary-heading">
        <div className="report-section-head">
          <Scale size={18} aria-hidden="true" />
          <h5 id="summary-heading">Model summary</h5>
        </div>
        <p>{explanation.summary}</p>
      </section>

      <section className="report-section input-context-section" aria-labelledby="input-context-heading">
        <div className="report-section-head">
          <ClipboardCheck size={18} aria-hidden="true" />
          <h5 id="input-context-heading">Input context</h5>
        </div>
        <dl className="context-list">
          <div>
            <dt>Event class</dt>
            <dd>{request.event.event_type ?? 'Unclassified'}</dd>
          </div>
          <div>
            <dt>Macro score</dt>
            <dd>{formatSignedNumber(request.facts.macro_context.total_macro_score)}</dd>
          </div>
          <div>
            <dt>Features</dt>
            <dd>{featureState}</dd>
          </div>
          <div>
            <dt>Prediction</dt>
            <dd>{predictionState}</dd>
          </div>
        </dl>
      </section>

      <section className="report-section" aria-labelledby="gates-heading">
        <div className="report-section-head">
          <ListChecks size={18} aria-hidden="true" />
          <h5 id="gates-heading">Risk gates</h5>
        </div>
        <div className="gate-list">
          {explanation.gates.map((gate) => (
            <div className={`gate-row ${gate.passed ? 'gate-pass' : 'gate-block'}`} key={gate.name}>
              {gate.passed ? (
                <CheckCircle2 size={17} aria-hidden="true" />
              ) : (
                <AlertTriangle size={17} aria-hidden="true" />
              )}
              <strong>{formatLabel(gate.name)}</strong>
              <span>{gate.reason}</span>
            </div>
          ))}
        </div>
      </section>

      <section className="report-section evidence-section" aria-labelledby="evidence-heading">
        <div className="report-section-head">
          <Database size={18} aria-hidden="true" />
          <h5 id="evidence-heading">Evidence</h5>
        </div>
        {explanation.evidence.length > 0 ? (
          <div className="evidence-list">
            {explanation.evidence.map((item) => (
              <div className="evidence-row" key={`${item.evidence_type}-${item.label}`}>
                <div>
                  <strong title={item.label}>{formatLabel(item.label)}</strong>
                  <span>{formatLabel(item.evidence_type)}</span>
                </div>
                <span>{formatSignedNumber(item.contribution)}</span>
                <span>{formatPercent(item.confidence)}</span>
              </div>
            ))}
          </div>
        ) : (
          <p className="quiet-copy">No evidence items returned.</p>
        )}
      </section>

      <section className="report-section utility-section" aria-labelledby="utility-heading">
        <div className="report-section-head">
          <Activity size={18} aria-hidden="true" />
          <h5 id="utility-heading">Action utilities</h5>
        </div>
        <div className="utility-list">
          {explanation.utilities.map((utility) => (
            <div className="utility-row" key={utility.action}>
              <span>{utility.action}</span>
              <div className="utility-track" aria-hidden="true">
                <span
                  className={utility.expected_utility >= 0 ? 'utility-positive' : 'utility-negative'}
                  style={{ width: utilityBarWidth(utility.expected_utility) }}
                />
              </div>
              <strong>{formatSignedNumber(utility.expected_utility)}</strong>
            </div>
          ))}
        </div>
      </section>

      <section className="report-section similar-events-section" aria-labelledby="similar-events-heading">
        <div className="report-section-head">
          <LineChart size={18} aria-hidden="true" />
          <h5 id="similar-events-heading">Similar-event history</h5>
        </div>
        {eventStudy ? (
          <dl className="context-list">
            <div>
              <dt>Samples</dt>
              <dd>{eventStudy.sample_count}</dd>
            </div>
            <div>
              <dt>CAR</dt>
              <dd>{formatSignedPercent(eventStudy.cumulative_abnormal_return)}</dd>
            </div>
            <div>
              <dt>Hit rate</dt>
              <dd>{eventStudy.hit_rate === null ? '-' : formatPercent(eventStudy.hit_rate)}</dd>
            </div>
            <div>
              <dt>T-stat</dt>
              <dd>{eventStudy.t_stat ?? '-'}</dd>
            </div>
          </dl>
        ) : (
          <p className="quiet-copy">No event-study evidence returned.</p>
        )}
      </section>

      <section className="report-section replay-section" aria-labelledby="replay-heading">
        <div className="report-section-head">
          <GitBranch size={18} aria-hidden="true" />
          <h5 id="replay-heading">Replay path</h5>
        </div>
        <ol className="pipeline-list">
          {explanation.pipeline.map((step) => (
            <li key={step}>{formatLabel(step)}</li>
          ))}
        </ol>
        <div className={missingFacts.length > 0 ? 'missing-facts' : 'missing-facts clear'}>
          <strong>{missingFacts.length > 0 ? 'Missing facts' : 'Missing facts clear'}</strong>
          <span>{missingFacts.length > 0 ? missingFacts.map(formatLabel).join(', ') : 'All required facts present'}</span>
        </div>
      </section>
    </div>
  )
}

function ImpactChart() {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const isDomTest = typeof navigator !== 'undefined' && navigator.userAgent.includes('jsdom')

  useEffect(() => {
    if (!containerRef.current || isDomTest) {
      return undefined
    }

    const chart = createChart(containerRef.current, {
      autoSize: true,
      height: 260,
      layout: {
        background: { color: '#ffffff' },
        textColor: '#3d4242',
      },
      grid: {
        vertLines: { color: '#eef0ef' },
        horzLines: { color: '#eef0ef' },
      },
      rightPriceScale: {
        borderColor: '#d7dcda',
      },
      timeScale: {
        borderColor: '#d7dcda',
      },
    })

    const line = chart.addSeries(LineSeries, {
      color: '#1f7a68',
      lineWidth: 2,
      priceLineVisible: false,
      lastValueVisible: false,
    })
    line.setData(impactSeries as LineData<string>[])

    return () => chart.remove()
  }, [isDomTest])

  if (isDomTest) {
    return (
      <div className="chart-fallback" role="img" aria-label="Market response chart">
        {impactSeries.map((point) => (
          <span key={point.time} style={{ height: `${Math.max(point.value * 100, 8)}%` }} />
        ))}
      </div>
    )
  }

  return (
    <>
      <div
        className="chart-accessible-summary"
        role="img"
        aria-label="Market response chart"
      >
        Event-study response curve with fixture data across seven market sessions.
      </div>
      <div className="impact-chart" ref={containerRef} />
    </>
  )
}

function formatCurrency(value: number | null): string {
  if (value === null) {
    return '-'
  }

  return new Intl.NumberFormat('en-IN', {
    maximumFractionDigits: 2,
    minimumFractionDigits: 0,
    style: 'currency',
    currency: 'INR',
  }).format(value)
}

function formatPercent(value: number): string {
  return new Intl.NumberFormat('en-IN', {
    maximumFractionDigits: 0,
    style: 'percent',
  }).format(value)
}

function buildEventFilterOptions(events: EventReviewSummary[]): EventFilterOptions {
  return {
    region: uniqueValues(events.map((event) => event.region)),
    market: uniqueValues(events.map((event) => event.symbol)),
    sector: uniqueValues(events.map((event) => event.sector)),
    eventClass: uniqueValues(events.map((event) => event.event_class)),
    source: uniqueValues(events.map((event) => event.source)),
    severity: uniqueValues(events.map((event) => event.severity)),
  }
}

function filterEvents(
  events: EventReviewSummary[],
  filters: EventFilterState,
): EventReviewSummary[] {
  return events.filter(
    (event) =>
      matchesFilter(event.region, filters.region) &&
      matchesFilter(event.symbol, filters.market) &&
      matchesFilter(event.sector, filters.sector) &&
      matchesFilter(event.event_class, filters.eventClass) &&
      matchesFilter(event.source, filters.source) &&
      matchesFilter(event.severity, filters.severity),
  )
}

function uniqueValues(values: Array<string | null>): string[] {
  return Array.from(new Set(values.filter((value): value is string => Boolean(value)))).sort(
    (left, right) => left.localeCompare(right),
  )
}

function matchesFilter(value: string | null, filter: string): boolean {
  return filter === 'all' || value === filter
}

function formatSignedPercent(value: number | null): string {
  if (value === null) {
    return '-'
  }

  return new Intl.NumberFormat('en-IN', {
    maximumFractionDigits: 1,
    signDisplay: 'always',
    style: 'percent',
  }).format(value)
}

function formatSignedNumber(value: number): string {
  return new Intl.NumberFormat('en-IN', {
    maximumFractionDigits: 4,
    signDisplay: 'always',
  }).format(value)
}

function formatLabel(value: string): string {
  return value
    .replaceAll('_', ' ')
    .toLowerCase()
    .replace(/\b\w/g, (letter) => letter.toUpperCase())
}

function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat('en-IN', {
    dateStyle: 'medium',
    timeStyle: 'short',
    timeZone: 'Asia/Kolkata',
  }).format(new Date(value))
}

function shortHash(value: string): string {
  return value.length > 12 ? `${value.slice(0, 8)}...${value.slice(-4)}` : value
}

function utilityBarWidth(value: number): string {
  return `${Math.min(Math.max(Math.abs(value) * 900, 6), 100)}%`
}

export default App
