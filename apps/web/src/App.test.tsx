import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import App from './App'

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
      return Response.json(
        decisionPayload ?? {
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
          reasons: [],
          execution_ready: true,
        },
      )
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
  expect(await screen.findByText('Online')).toBeInTheDocument()
})

test('posts the smoke fixture and renders the backend decision', async () => {
  const fetchMock = mockFetch()
  renderApp()

  await userEvent.click(screen.getByRole('button', { name: /run fixture/i }))

  expect(await screen.findByRole('heading', { name: 'BUY' })).toBeInTheDocument()
  expect(screen.getByText('Execution ready')).toBeInTheDocument()
  expect(screen.getByText('72%')).toBeInTheDocument()
  expect(screen.getByText('20')).toBeInTheDocument()
  expect(screen.getByText('₹1,030')).toBeInTheDocument()
  expect(fetchMock).toHaveBeenCalledWith(
    'http://127.0.0.1:8000/decide',
    expect.objectContaining({ method: 'POST' }),
  )
})

