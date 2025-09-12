// Basic smoke tests for REST, Web UI, and Metrics
// Run with: npm test

const { test, expect } = require('@playwright/test')

const REST = 'http://localhost:8080'
const WEB = 'http://localhost:8090'
const METRICS = 'http://localhost:9090'

// REST health endpoints
test('REST /healthz returns healthy', async ({ request }) => {
  const res = await request.get(`${REST}/healthz`, { timeout: 5000 })
  expect(res.ok()).toBeTruthy()
  const json = await res.json()
  expect(json.status).toBe('healthy')
})

test('REST /health/live and /health/ready return OK', async ({ request }) => {
  const live = await request.get(`${REST}/health/live`, { timeout: 5000 })
  expect(live.ok()).toBeTruthy()
  const ready = await request.get(`${REST}/health/ready`, { timeout: 5000 })
  expect(ready.ok()).toBeTruthy()
})

// Web management
test('WEB /healthz returns web-server healthy', async ({ request }) => {
  const res = await request.get(`${WEB}/healthz`, { timeout: 5000 })
  expect(res.ok()).toBeTruthy()
  const json = await res.json()
  expect(json.service).toBe('web-server')
  expect(json.status).toBe('healthy')
})

test('WEB / renders SPA root', async ({ page }) => {
  await page.goto(`${WEB}/`, { waitUntil: 'domcontentloaded', timeout: 5000 })
  // 应存在 SPA 根节点
  await expect(page.locator('#app')).toHaveCount(1)
  // 标题包含 Gateway 字样（更宽松）
  await expect(page).toHaveTitle(/Gateway/i)
})

// Metrics
test('METRICS /metrics returns Prometheus text', async ({ request }) => {
  const res = await request.get(`${METRICS}/metrics`, { timeout: 5000 })
  expect(res.ok()).toBeTruthy()
  const txt = await res.text()
  expect(txt.length).toBeGreaterThan(100)
})
