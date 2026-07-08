import { expect, test } from '@playwright/test'

test('submits the decision fixture and renders the evidence audit trail', async ({ page }) => {
  await page.goto('/')

  await expect(page.getByText('Online')).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Normalized review queue' })).toBeVisible()
  await expect(page.getByText('Quarterly earnings beat estimates', { exact: true }).first()).toBeVisible()

  await page.getByLabel('Event class').selectOption('MEDICAL_CLASSIFICATION')

  await expect(
    page.getByText('Therapy classification update affects reimbursement basket', { exact: true }).first(),
  ).toBeVisible()
  await expect(page.getByText('ICD-11 respiratory classification')).toBeVisible()
  await expect(page.getByText('raw-who-icd11')).toBeVisible()

  await page.getByRole('button', { name: /run fixture/i }).click()

  await expect(page.getByRole('heading', { name: 'BUY' })).toBeVisible()
  await expect(page.getByText('Paper ready')).toBeVisible()
  await expect(page.getByText('rules-impact-v1').first()).toBeVisible()
  await expect(page.getByLabel('Decision replay metadata')).toContainText('Input hash')
  await expect(page.getByLabel('Decision replay metadata')).toContainText('norm-smoke-earnings v1')

  await expect(page.getByRole('heading', { name: 'Risk gates' })).toBeVisible()
  await expect(page.getByText('Evidence', { exact: true }).first()).toBeVisible()
  await expect(page.getByText('Price', { exact: true }).first()).toBeVisible()
  await expect(page.getByText('Confidence', { exact: true }).first()).toBeVisible()

  await expect(page.getByRole('heading', { name: 'Evidence', exact: true })).toBeVisible()
  await expect(page.getByText('Earnings Beat', { exact: true })).toBeVisible()
  await expect(page.getByText('Car Fixture', { exact: true })).toBeVisible()

  await expect(page.getByRole('heading', { name: 'Input context' })).toBeVisible()
  await expect(page.getByText('EARNINGS', { exact: true })).toBeVisible()
  await expect(page.getByText('Not supplied').first()).toBeVisible()

  await expect(page.getByRole('heading', { name: 'Similar-event history' })).toBeVisible()
  await expect(page.getByText('Missing facts clear')).toBeVisible()
  await expect(page.getByText('PAPER', { exact: true })).toBeVisible()
  await expect(page.getByRole('img', { name: 'Market response chart' })).toBeAttached()
})
