import { expect, test } from '@playwright/test'

const UI_DECISION_P95_MAX_MS = Number(process.env.UI_DECISION_P95_MAX_MS ?? '2500')

test('decision workflow p95 stays inside the release budget', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText('Online')).toBeVisible()

  const samples: number[] = []

  for (let index = 0; index < 8; index += 1) {
    const startedAt = performance.now()

    await Promise.all([
      page.waitForResponse(
        (response) => response.url().endsWith('/decide') && response.status() === 200,
      ),
      page.getByRole('button', { name: /run fixture|running/i }).click(),
    ])
    await expect(page.getByRole('heading', { name: 'BUY' })).toBeVisible()

    samples.push(performance.now() - startedAt)
  }

  samples.sort((left, right) => left - right)
  const p95 = samples[Math.ceil(samples.length * 0.95) - 1] ?? 0

  expect(
    p95,
    `UI decision p95 ${Math.round(p95)}ms exceeded ${UI_DECISION_P95_MAX_MS}ms; samples=${samples
      .map((sample) => Math.round(sample))
      .join(',')}`,
  ).toBeLessThanOrEqual(UI_DECISION_P95_MAX_MS)
})
