import type { DecisionRequest } from './api'

export const earningsDecisionRequest: DecisionRequest = {
  event: {
    event_id: 'norm-smoke-earnings',
    version: 1,
    causal_parent_id: 'raw-smoke-earnings',
    event_type: 'EARNINGS',
    headline: 'Quarterly earnings beat estimates',
    body: 'Profit rose and revenue grew higher than expected.',
    occurred_at: '2026-07-06T09:15:00Z',
    symbol: 'RELIANCE',
    sector: 'Oil & Gas',
    source: 'NSE',
    region: 'IN',
    impact_level: null,
    impact_category: null,
  },
  facts: {
    macro_context: {
      sp500_futures_change: 0,
      nasdaq_futures_change: 0,
      brent_crude_change: 0,
      usd_inr_change: 0,
      fii_net_flow: 0,
      gold_change: 0,
      total_macro_score: 0,
    },
    entry_price: 1000,
    exchange: 'NSE',
    features: null,
    prediction: null,
    event_study: { abnormal_returns: [0.012, 0.018, -0.004, 0.021, 0.009] },
    kg_modifier: 0,
  },
}

export const impactSeries = [
  { time: '2026-07-01', value: 0.0 },
  { time: '2026-07-02', value: 0.18 },
  { time: '2026-07-03', value: 0.36 },
  { time: '2026-07-06', value: 0.72 },
  { time: '2026-07-07', value: 0.54 },
  { time: '2026-07-08', value: 0.61 },
  { time: '2026-07-09', value: 0.48 },
]
