import { test, expect } from '@playwright/test'

// Increase default timeout to 120s to accommodate server warm-up in CI/containers
test.setTimeout(120_000)
import WebSocket from 'ws'

// Base URLs (can be overridden by env)
const REST_BASE = process.env.REST_BASE || 'http://localhost:18080'
const METRICS_BASE = process.env.METRICS_BASE || 'http://localhost:19090'
const SWAGGER_UI_CANDIDATES = [
  `${REST_BASE}/docs/`,
  `${REST_BASE}/docs/swagger-ui/`,
]
const OPENAPI_JSON = `${REST_BASE}/docs/openapi.json`
const SYS_HEALTH = `${REST_BASE}/system/health`
const SYS_HEALTH_V1 = `${REST_BASE}/api/v1/system/health`
const WS_URL = (process.env.WS_URL || REST_BASE.replace('http', 'ws')) + '/ws/telemetry'

// Utilities
async function waitUntilOk(url: string, timeoutMs = 60_000) {
  const start = Date.now()
  let lastStatus = 0
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url)
      lastStatus = res.status
      if (res.ok) return res
    } catch {
      // ignore and retry
    }
    await new Promise(r => setTimeout(r, 1000))
  }
  throw new Error(`Timeout waiting for ${url}, lastStatus=${lastStatus}`)
}

async function firstAvailableUrl(urls: string[], timeoutMs = 30_000): Promise<string> {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    for (const url of urls) {
      try {
        const res = await fetch(url)
        if (res.ok) return url
      } catch {}
    }
    await new Promise(r => setTimeout(r, 1000))
  }
  throw new Error(`No available URL among: ${urls.join(', ')}`)
}

// Smoke: OpenAPI JSON
test('OpenAPI JSON is reachable and valid-ish', async () => {
  const res = await waitUntilOk(OPENAPI_JSON, 120_000)
  expect(res.ok).toBeTruthy()
  expect(res.headers.get('content-type') || '').toMatch(/json/i)
  const json = await res.json()
  // 'openapi' field or at least 'info' should exist
  expect(json.openapi || json.info).toBeTruthy()
})

// Smoke: System health endpoints
test('System health returns JSON (root and v1)', async () => {
  const res1 = await waitUntilOk(SYS_HEALTH, 120_000)
  expect(res1.ok).toBeTruthy()
  const j1 = await res1.json()
  expect(typeof j1).toBe('object')
  // keys are not strictly required, but commonly include postgres/influxdb
  expect(Object.keys(j1).length).toBeGreaterThan(0)

  const res2 = await waitUntilOk(SYS_HEALTH_V1, 120_000)
  expect(res2.ok).toBeTruthy()
  const j2 = await res2.json()
  expect(typeof j2).toBe('object')
})

// Smoke: Swagger UI page renders (HTML available)
test('Swagger UI page loads', async () => {
  const url = await firstAvailableUrl(SWAGGER_UI_CANDIDATES, 120_000)
  const res = await waitUntilOk(url, 120_000)
  expect(res.ok).toBeTruthy()
  const html = await res.text()
  // The fallback HTML always includes this container; utoipa UI also does
  expect(html).toMatch(/id=["']swagger-ui["']/i)
})

// Optional: WebSocket handshake
test('WebSocket /ws/telemetry accepts handshake', async () => {
  test.skip(process.env.SKIP_WS === '1', 'Skipping WS handshake by env')

  const ws = new WebSocket(WS_URL, { handshakeTimeout: 5000 })
  await new Promise<void>((resolve, reject) => {
    const to = setTimeout(() => reject(new Error('WS timeout')), 7000)
    ws.once('open', () => { clearTimeout(to); ws.close(); resolve() })
    ws.once('error', err => { clearTimeout(to); reject(err) })
  })
})
