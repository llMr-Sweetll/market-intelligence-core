import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import App from './App'

const defaultDecisionPayload = {
  decision_id: '960de912-3818-5207-a7cd-e1efb52b62c1',
  parent_event_id: 'norm-smoke-earnings',
  parent_event_version: 1,
  action: 'BUY',
  total_score: 0.72,
  confidence: 0.72,
  position_size: 0.02,
  quantity: 20,
  entry_price: 1000,
  target_price: 1030,
  stop_loss: 980,
  timing: 'Immediate market order',
  exchange: 'NSE',
  symbol: 'RELIANCE',
  sector: 'Oil & Gas',
  thesis: 'BUY RELIANCE with quantity 20 near 1000. Target 1030 and stop 980.',
  reasons: [{ rule_id: 'earnings_positive', contribution: 0.72, rationale: 'earnings surprise' }],
  model_version: 'rules-impact-v1',
  input_hash: '09f2b77a-6e99-5aa1-9ae9-7dd74a83bc11',
  expected_return: 0.021,
  downside: -0.0305,
  explanation: {
    model_version: 'rules-impact-v1',
    input_hash: '09f2b77a-6e99-5aa1-9ae9-7dd74a83bc11',
    pipeline: [
      'classify_event',
      'resolve_entities',
      'collect_evidence',
      'estimate_impact',
      'apply_risk_liquidity_confidence_gates',
      'decide',
    ],
    entity_resolution: {
      symbol: 'RELIANCE',
      sector: 'Oil & Gas',
      region: 'IN',
      confidence: 0.95,
    },
    evidence: [
      {
        evidence_type: 'rule',
        label: 'earnings_positive',
        contribution: 0.72,
        confidence: 0.9,
      },
      {
        evidence_type: 'event_study',
        label: 'car_fixture',
        contribution: 0.2,
        confidence: 0.68,
      },
    ],
    impact: {
      combined_score: 0.72,
      expected_return: 0.021,
      downside: -0.0305,
      event_study: {
        sample_count: 5,
        cumulative_abnormal_return: 0.056,
        mean_abnormal_return: 0.0112,
        hit_rate: 0.8,
        t_stat: 2.1,
        calibrated_weight: 0.2,
        calibrated_confidence: 0.68,
      },
    },
    gates: [
      { name: 'evidence', passed: true, reason: '2 evidence item(s) available' },
      { name: 'price', passed: true, reason: 'as-of entry price supplied' },
      { name: 'confidence', passed: true, reason: 'confidence 0.72' },
    ],
    utilities: [
      { action: 'BUY', expected_utility: 0.003 },
      { action: 'SELL', expected_utility: -0.0456 },
      { action: 'HOLD', expected_utility: 0 },
      { action: 'PAPER', expected_utility: 0.0023 },
    ],
    recommended_action: 'BUY',
    confidence: 0.72,
    missing_facts: [],
    summary: 'BUY via rules-impact-v1 for RELIANCE. Event class Earnings, score 0.72, confidence 0.72.',
  },
  execution_ready: true,
}

function renderApp() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>,
  )
}

function mockFetch(decisionPayload?: unknown) {
  const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
    const url = input.toString()

    if (url.endsWith('/health')) {
      return Response.json({ status: 'ok', service: 'gm-api' })
    }

    if (url.endsWith('/decide')) {
      return Response.json(decisionPayload ?? defaultDecisionPayload)
    }

    return new Response('not found', { status: 404 })
  })

  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

test('renders the operator console with live health state', async () => {
  mockFetch()
  renderApp()

  expect(screen.getByRole('heading', { name: 'Command Center' })).toBeInTheDocument()
  expect(screen.getByRole('heading', { name: 'Normalized review queue' })).toBeInTheDocument()
  expect(screen.getByRole('heading', { name: 'Evidence and audit trail' })).toBeInTheDocument()
  expect(await screen.findByText('Online')).toBeInTheDocument()
})

test('posts the smoke fixture and renders the backend decision', async () => {
  const fetchMock = mockFetch()
  renderApp()

  await userEvent.click(screen.getByRole('button', { name: /run fixture/i }))

  expect(await screen.findByRole('heading', { name: 'BUY' })).toBeInTheDocument()
  expect(screen.getByText('Paper ready')).toBeInTheDocument()
  expect(screen.getByText('72%')).toBeInTheDocument()
  expect(screen.getByText('20')).toBeInTheDocument()
  expect(screen.getByText('₹1,030')).toBeInTheDocument()
  expect(screen.getByText('rules-impact-v1')).toBeInTheDocument()
  expect(screen.getByText('09f2b77a...bc11')).toBeInTheDocument()
  expect(screen.getByText('Earnings Positive')).toBeInTheDocument()
  expect(screen.getByText('Car Fixture')).toBeInTheDocument()
  expect(screen.getByText('EARNINGS')).toBeInTheDocument()
  expect(screen.getByText('Similar-event history')).toBeInTheDocument()
  expect(screen.getByText('Missing facts clear')).toBeInTheDocument()
  expect(screen.getByText('PAPER')).toBeInTheDocument()
  expect(fetchMock).toHaveBeenCalledWith(
    'http://127.0.0.1:8000/decide',
    expect.objectContaining({ method: 'POST' }),
  )
})
