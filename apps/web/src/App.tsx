import { useEffect, useMemo, useRef } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import {
  Activity,
  ArrowUpRight,
  Banknote,
  Bell,
  Building2,
  CheckCircle2,
  CircleDollarSign,
  Database,
  GitBranch,
  Globe2,
  Landmark,
  LineChart,
  Network,
  PlugZap,
  RefreshCw,
  ShieldCheck,
  Stethoscope,
  TrendingUp,
} from 'lucide-react'
import { createChart, LineSeries, type LineData } from 'lightweight-charts'

import {
  createDecision,
  fetchHealth,
  getApiBaseUrl,
  type Decision,
  type HealthResponse,
} from './api'
import { earningsDecisionRequest, impactSeries } from './fixtures'

type StatusTone = 'green' | 'amber' | 'red' | 'blue' | 'neutral'

type Metric = {
  label: string
  value: string
  detail: string
  tone: StatusTone
}

type EventRow = {
  id: string
  title: string
  source: string
  category: string
  symbol: string
  priority: string
}

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

const metrics: Metric[] = [
  { label: 'API status', value: 'Live check', detail: 'GET /health', tone: 'blue' },
  { label: 'Decision path', value: 'Deterministic', detail: 'No clock or random input', tone: 'green' },
  { label: 'Order mode', value: 'Disabled', detail: 'Read-only plus paper trading', tone: 'amber' },
  { label: 'Payments', value: 'Test mode', detail: 'Webhook verification planned', tone: 'neutral' },
]

const eventRows: EventRow[] = [
  {
    id: 'evt-earnings',
    title: 'Quarterly earnings beat estimates',
    source: 'NSE',
    category: 'Earnings',
    symbol: 'RELIANCE',
    priority: 'High',
  },
  {
    id: 'evt-policy',
    title: 'Central bank policy statement updates liquidity stance',
    source: 'Regulator',
    category: 'Policy',
    symbol: 'BANKNIFTY',
    priority: 'Review',
  },
  {
    id: 'evt-medical',
    title: 'Therapy classification update affects reimbursement basket',
    source: 'Registry',
    category: 'Medical',
    symbol: 'PHARMA',
    priority: 'Watch',
  },
  {
    id: 'evt-corporate',
    title: 'Company board approves structure and reporting change',
    source: 'Filing',
    category: 'Company',
    symbol: 'NIFTY50',
    priority: 'Review',
  },
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
    detail: 'Checkout and signed webhook verification before public release.',
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
  const health = useQuery<HealthResponse>({
    queryKey: ['health'],
    queryFn: fetchHealth,
    retry: false,
    refetchInterval: 30_000,
  })

  const decision = useMutation<Decision, Error, void>({
    mutationFn: () => createDecision(earningsDecisionRequest),
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
            <button className="ghost-button" type="button">
              <Database size={17} aria-hidden="true" />
              Import feed
            </button>
          </div>

          <div className="event-table" role="table" aria-label="Normalized events">
            <div className="event-table-head" role="row">
              <span role="columnheader">Event</span>
              <span role="columnheader">Source</span>
              <span role="columnheader">Type</span>
              <span role="columnheader">Symbol</span>
              <span role="columnheader">Priority</span>
            </div>
            {eventRows.map((event) => (
              <div className="event-row" role="row" key={event.id}>
                <strong role="cell">{event.title}</strong>
                <span role="cell">{event.source}</span>
                <span role="cell">{event.category}</span>
                <span role="cell">{event.symbol}</span>
                <span className="priority" role="cell">
                  {event.priority}
                </span>
              </div>
            ))}
          </div>
        </section>

        <section
          id="decision"
          className="panel decision-panel"
          aria-labelledby="decision-heading"
        >
          <div className="section-heading">
            <div>
              <p className="eyebrow">Decision Workbench</p>
              <h3 id="decision-heading">Backend contract smoke path</h3>
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
              </dl>
            </div>

            <DecisionResult decision={decision.data} error={decision.error} />
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

        <section className="panel payment-panel" aria-labelledby="payments-heading">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Payments</p>
              <h3 id="payments-heading">Commercial gate plan</h3>
            </div>
            <CircleDollarSign size={22} aria-hidden="true" />
          </div>

          <div className="payment-grid">
            <div>
              <Banknote size={20} aria-hidden="true" />
              <strong>Razorpay test mode</strong>
              <span>Checkout, subscription state, and signed webhooks before billing is enabled.</span>
            </div>
            <div>
              <CheckCircle2 size={20} aria-hidden="true" />
              <strong>Access state</strong>
              <span>Paid features can be gated without changing deterministic decision behavior.</span>
            </div>
            <div>
              <ArrowUpRight size={20} aria-hidden="true" />
              <strong>Release switch</strong>
              <span>Production keys stay out of the repository and are verified through environment checks.</span>
            </div>
          </div>
        </section>
      </main>
    </div>
  )
}

function DecisionResult({
  decision,
  error,
}: {
  decision: Decision | undefined
  error: Error | null
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
          {decision.execution_ready ? 'Execution ready' : 'Review'}
        </span>
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
          <dt>Stop</dt>
          <dd>{formatCurrency(decision.stop_loss)}</dd>
        </div>
        <div>
          <dt>ID</dt>
          <dd className="mono">{decision.decision_id.slice(0, 8)}</dd>
        </div>
      </dl>

      <p className="thesis">{decision.thesis}</p>
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

  return <div className="impact-chart" ref={containerRef} role="img" aria-label="Market response chart" />
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

export default App
