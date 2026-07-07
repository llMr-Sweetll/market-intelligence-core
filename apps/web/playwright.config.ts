import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig, devices } from '@playwright/test'

const appDir = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(appDir, '../..')
const apiPort = process.env.PW_API_PORT ?? '18101'
const webPort = process.env.PW_WEB_PORT ?? '5174'
const apiUrl = `http://127.0.0.1:${apiPort}`
const webUrl = `http://127.0.0.1:${webPort}`

export default defineConfig({
  testDir: './tests/e2e',
  timeout: 60_000,
  expect: {
    timeout: 12_000,
  },
  use: {
    baseURL: webUrl,
    trace: 'retain-on-failure',
  },
  webServer: [
    {
      command: `cargo run -p gm-api -- --host 127.0.0.1 --port ${apiPort}`,
      cwd: repoRoot,
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
      url: `${apiUrl}/health`,
    },
    {
      command: `VITE_API_BASE_URL=${apiUrl} npm run dev -- --host 127.0.0.1 --port ${webPort}`,
      cwd: appDir,
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
      url: webUrl,
    },
  ],
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'mobile-chrome',
      use: { ...devices['Pixel 7'] },
    },
  ],
})
