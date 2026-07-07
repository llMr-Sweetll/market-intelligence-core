export type NormalizedEvent = {
  event_id: string
  version: number
  causal_parent_id: string | null
  event_type: string | null
  headline: string
  body: string
  occurred_at: string
  symbol: string | null
  sector: string | null
  source: string | null
  region: string | null
  impact_level: string | null
  impact_category: string | null
}

export type MacroContext = {
  sp500_futures_change: number
  nasdaq_futures_change: number
  brent_crude_change: number
  usd_inr_change: number
  fii_net_flow: number
  gold_change: number
  total_macro_score: number
}

export type DecisionRequest = {
  event: NormalizedEvent
  facts: {
    macro_context: MacroContext
    entry_price: number | null
    exchange: string | null
    features: unknown | null
    prediction: unknown | null
    event_study: { abnormal_returns: number[] } | null
    kg_modifier: number
  }
}

export type HealthResponse = {
  status: string
  service: string
}

export type DecisionAction = 'BUY' | 'SELL' | 'HOLD'

export type CandidateAction = DecisionAction | 'PAPER'

export type EvidenceItem = {
  evidence_type: string
  label: string
  contribution: number
  confidence: number
}

export type EventStudySummary = {
  sample_count: number
  cumulative_abnormal_return: number | null
  mean_abnormal_return: number | null
  hit_rate: number | null
  t_stat: number | null
  calibrated_weight: number
  calibrated_confidence: number
}

export type ImpactEstimate = {
  combined_score: number
  expected_return: number
  downside: number
  event_study: EventStudySummary | null
}

export type GateReport = {
  name: string
  passed: boolean
  reason: string
}

export type UtilityEstimate = {
  action: CandidateAction
  expected_utility: number
}

export type ModelExplanation = {
  model_version: string
  input_hash: string
  pipeline: string[]
  entity_resolution: {
    symbol: string | null
    sector: string | null
    region: string | null
    confidence: number
  }
  evidence: EvidenceItem[]
  impact: ImpactEstimate
  gates: GateReport[]
  utilities: UtilityEstimate[]
  recommended_action: DecisionAction
  confidence: number
  missing_facts: string[]
  summary: string
}

export type Decision = {
  decision_id: string
  parent_event_id: string
  parent_event_version: number
  action: DecisionAction
  total_score: number
  confidence: number
  position_size: number
  quantity: number | null
  entry_price: number | null
  target_price: number | null
  stop_loss: number | null
  timing: string | null
  exchange: string | null
  symbol: string | null
  sector: string | null
  thesis: string
  reasons: Array<{ rule_id?: string; contribution?: number; rationale?: string }>
  model_version: string
  input_hash: string
  expected_return: number | null
  downside: number | null
  explanation: ModelExplanation
  execution_ready: boolean
}

const apiBaseUrl = (import.meta.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:8000').replace(
  /\/$/,
  '',
)

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBaseUrl}${path}`, {
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
    ...init,
  })

  if (!response.ok) {
    const body = await response.text()
    throw new Error(`${response.status} ${response.statusText}: ${body}`)
  }

  return (await response.json()) as T
}

export function fetchHealth(): Promise<HealthResponse> {
  return request<HealthResponse>('/health')
}

export function createDecision(payload: DecisionRequest): Promise<Decision> {
  return request<Decision>('/decide', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export function getApiBaseUrl(): string {
  return apiBaseUrl
}
