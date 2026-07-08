import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'

test('primary operator screens have no serious accessibility violations', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText('Online')).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Normalized review queue' })).toBeVisible()

  await page.getByLabel('Event class').selectOption('MEDICAL_CLASSIFICATION')
  await expect(page.getByText('ICD-11 respiratory classification')).toBeVisible()

  await page.getByRole('button', { name: /run fixture/i }).click()
  await expect(page.getByRole('heading', { name: 'BUY' })).toBeVisible()
  await expect(page.getByRole('img', { name: 'Market response chart' })).toBeAttached()

  const results = await new AxeBuilder({ page })
    .include('main')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .analyze()

  const blockingViolations = results.violations.filter((violation) =>
    ['critical', 'serious'].includes(violation.impact ?? ''),
  )

  expect(blockingViolations).toEqual([])
})
