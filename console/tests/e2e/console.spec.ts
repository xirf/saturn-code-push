import { createHash } from 'node:crypto'
import { test, expect, type APIRequestContext, type Page } from '@playwright/test'

const baseURL = process.env.E2E_BASE_URL ?? 'http://saturn-e2e-server:3000'
const token = process.env.E2E_ADMIN_TOKEN
const password = process.env.E2E_PASSWORD

if (!token || !password) {
  throw new Error(
    'Set isolated E2E_ADMIN_TOKEN and E2E_PASSWORD values before running browser tests',
  )
}

async function signIn(page: Page) {
  test.setTimeout(90_000)
  await page.goto('/login')
  await page.screenshot({ path: 'test-results/login-desktop.png', fullPage: true })
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  const responsePromise = page.waitForResponse(
    (response) => new URL(response.url()).pathname === '/auth/login',
  )
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  const response = await responsePromise
  if (response.status() === 429) {
    await page.waitForTimeout(Math.min(60, Number(response.headers()['retry-after']) || 60) * 1000)
    await page.getByLabel('Password', { exact: true }).fill(password!)
    await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  }
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible()
}

async function seedRelease(api: APIRequestContext, name = 'Browser lifecycle fixture') {
  const headers = { Authorization: `Bearer ${token}` }
  const appResponse = await api.post('/api/v1/apps', {
    headers,
    data: { display_name: name },
  })
  expect(appResponse.ok()).toBeTruthy()
  const app = await appResponse.json()
  const releaseResponse = await api.post(`/api/v1/apps/${app.id}/releases`, {
    headers,
    data: { version: '1.0.0+1', flutter_revision: 'browser-fixture' },
  })
  const { release } = await releaseResponse.json()
  const bytes = Buffer.from('abc')
  const metadata = {
    arch: 'x86_64',
    platform: 'android',
    hash: createHash('sha256').update(bytes).digest('hex'),
    size: '3',
  }

  async function upload(path: string) {
    const registration = await api.post(path, { headers, multipart: metadata })
    expect(registration.ok()).toBeTruthy()
    const artifact = await registration.json()
    const result = await api.post(new URL(artifact.url).pathname, {
      multipart: {
        file: { name: 'artifact', mimeType: 'application/octet-stream', buffer: bytes },
      },
    })
    expect(result.ok()).toBeTruthy()
  }

  await upload(`/api/v1/apps/${app.id}/releases/${release.id}/artifacts`)
  const activate = await api.patch(`/api/v1/apps/${app.id}/releases/${release.id}`, {
    headers,
    data: { platform: 'android', status: 'active' },
  })
  expect(activate.ok()).toBeTruthy()
  const patchResponse = await api.post(`/api/v1/apps/${app.id}/patches`, {
    headers,
    data: { release_id: release.id },
  })
  const patch = await patchResponse.json()
  await upload(`/api/v1/apps/${app.id}/patches/${patch.id}/artifacts`)
  await api.post('/api/v1/patches/events', {
    data: {
      event: {
        app_id: app.id,
        type: '__patch_download__',
        client_id: 'browser-fixture',
        release_version: release.version,
        patch_number: patch.number,
      },
    },
  })
  return { app, release, patch }
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const violations: string[] = []
    Object.defineProperty(window, '__cspViolations', { value: violations })
    document.addEventListener('securitypolicyviolation', (event) => {
      violations.push(`${event.violatedDirective}: ${event.blockedURI}`)
    })
  })
})

test.afterEach(async ({ page }) => {
  const violations = await page.evaluate(() => Reflect.get(window, '__cspViolations') ?? [])
  expect(violations, 'The deployed console must not violate the server CSP').toEqual([])
})

test('sign-in, application creation, direct links, and static asset security', async ({ page }) => {
  const pageErrors: string[] = []
  const requestOrigins = new Set<string>()
  page.on('pageerror', (error) => pageErrors.push(error.message))
  page.on('request', (request) => requestOrigins.add(new URL(request.url()).origin))
  await page.goto('/new')
  await expect(page).toHaveURL(/\/login$/)
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill('incorrect-password')
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Incorrect username or password')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'New application', exact: true }).click()
  await expect(page).toHaveURL(/\/new$/)
  await page.getByLabel('Application name').fill('Created in browser')
  await page.getByRole('button', { name: 'Create application', exact: true }).click()
  // Creating an application lands inside it, on its getting-started overview.
  await expect(page).toHaveURL(/\/apps\/[^/]+$/)
  await expect(page.getByRole('heading', { name: 'Created in browser', exact: true })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Get started', exact: true })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Created in browser', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'All applications', exact: true }).click()
  await expect(page).toHaveURL(/\/$/)
  await expect(page.getByText('Created in browser', { exact: true }).first()).toBeVisible()
  expect(await page.evaluate(() => ({ ...localStorage, ...sessionStorage }))).toEqual({})
  expect(pageErrors).toEqual([])
  expect([...requestOrigins]).toEqual([new URL(baseURL).origin])
  await page.screenshot({ path: 'test-results/applications-desktop.png', fullPage: true })
})

test('nested release routes and patch promotion, rollback, and restore', async ({
  page,
  request,
}) => {
  const { app, release } = await seedRelease(request)
  await signIn(page)
  await page.setViewportSize({ width: 1920, height: 1080 })
  await expect(page.getByRole('button', { name: 'Refresh workspace' })).toHaveCount(0)
  expect(await page.title()).not.toMatch(/[\u00c2\u00c3]/)
  // The sidebar is workspace-scoped until an application is opened.
  await expect(
    page
      .getByRole('navigation', { name: 'Main navigation' })
      .getByRole('link', { name: 'Patches' }),
  ).toHaveCount(0)
  await page.getByRole('button', { name: 'Switch application' }).click()
  const applicationPicker = page.getByRole('menu')
  await expect(applicationPicker).toBeVisible()
  const pickerBounds = await applicationPicker.boundingBox()
  expect(pickerBounds?.height).toBeLessThan(600)
  await page.screenshot({ path: 'test-results/application-picker-desktop.png' })
  await page.getByRole('menuitem', { name: app.display_name, exact: true }).click()
  await expect(page).toHaveURL(new RegExp(`/apps/${app.id}$`))
  await expect(page.getByRole('heading', { name: app.display_name, exact: true })).toBeVisible()
  const overviewChart = page.getByTestId('activity-chart')
  await expect
    .poll(async () =>
      overviewChart
        .locator('svg')
        .first()
        .evaluate((element) => element.getBoundingClientRect().width),
    )
    .toBeGreaterThan(900)
  await page.screenshot({ path: 'test-results/overview-desktop.png', fullPage: true })
  await page
    .getByRole('navigation', { name: 'Main navigation' })
    .getByRole('link', { name: 'Patches', exact: true })
    .click()
  await page.getByRole('button', { name: 'Promote', exact: true }).click()
  let dialog = page.getByRole('dialog')
  await expect(dialog.getByRole('heading')).toContainText('Promote patch #1')
  await dialog.getByRole('button', { name: 'Promote patch', exact: true }).click()
  await expect(dialog).not.toBeVisible()
  await expect(page.getByText('Published', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Actions for patch 1' }).click()
  const menu = page.getByRole('menu')
  await expect(menu).toBeVisible()
  const bounds = await menu.boundingBox()
  expect(bounds?.height).toBeLessThan(120)
  await page.screenshot({ path: 'test-results/patch-actions-desktop.png' })
  await page.getByRole('menuitem', { name: 'Roll back patch' }).click()
  dialog = page.getByRole('dialog')
  await dialog.getByRole('button', { name: 'Roll back patch', exact: true }).click()
  await expect(dialog).not.toBeVisible()
  await expect(page.getByText('Rolled back', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Actions for patch 1' }).click()
  await page.getByRole('menuitem', { name: 'Restore patch' }).click()
  await page.getByRole('dialog').getByRole('button', { name: 'Restore patch', exact: true }).click()
  await expect(page.getByText('Published', { exact: true })).toBeVisible()
  await page.goto(`/apps/${app.id}/releases/${release.id}`)
  await expect(page.getByText('browser-fixture', { exact: false }).first()).toBeVisible()
  await expect(page.getByRole('cell', { name: 'x86_64', exact: true })).toBeVisible()
  await page.setViewportSize({ width: 390, height: 844 })
  await page.getByRole('button', { name: 'Open navigation' }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.getByRole('dialog').getByRole('link', { name: 'Analytics', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Analytics', exact: true })).toBeVisible()
  const chart = page.getByTestId('activity-chart').locator('svg').first()
  await expect(chart).toBeVisible()
  const chartBounds = await chart.boundingBox()
  expect(chartBounds?.height).toBeGreaterThanOrEqual(240)
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
  ).toBeTruthy()
  await page.screenshot({ path: 'test-results/analytics-mobile.png', fullPage: true })
})

test('logout clears private data and cancels a delayed response before the next login', async ({
  page,
}) => {
  await signIn(page)
  let releaseResponse!: () => void
  let responseStarted!: () => void
  const gate = new Promise<void>((resolve) => {
    releaseResponse = resolve
  })
  const started = new Promise<void>((resolve) => {
    responseStarted = resolve
  })
  await page.route(
    '**/admin/state',
    async (route) => {
      const response = await route.fetch()
      const payload = await response.json()
      payload.apps[0].display_name = 'STALE_PRIVATE_DATA'
      responseStarted()
      await gate
      await route.fulfill({ json: payload }).catch(() => {})
    },
    { times: 1 },
  )
  await page.reload({ waitUntil: 'domcontentloaded' })
  await started
  await page.getByRole('button', { name: 'Sign out', exact: true }).click()
  await expect(page).toHaveURL(/\/login$/)
  releaseResponse()
  await signIn(page)
  await expect(page.getByText('STALE_PRIVATE_DATA', { exact: true })).not.toBeVisible()
  expect(await page.evaluate(() => ({ ...localStorage, ...sessionStorage }))).toEqual({})
})

test('an expired session returns to sign-in without showing cached workspace data', async ({
  page,
  context,
}) => {
  await signIn(page)
  const response = await context.request.post(`${baseURL}/auth/logout`, {
    headers: { Origin: new URL(baseURL).origin },
    data: {},
  })
  expect(response.ok()).toBeTruthy()
  await page.reload({ waitUntil: 'domcontentloaded' })
  await expect(page).toHaveURL(/\/login$/)
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).not.toBeVisible()
  await page.goto('/patches')
  await expect(page).toHaveURL(/\/login$/)
})

test('asset bundles verify uploads and follow patch delivery, rollback, and restore', async ({
  page,
  request,
}) => {
  const { app, release, patch } = await seedRelease(request, 'Asset delivery fixture')
  const headers = { Authorization: `Bearer ${token}` }
  const appPath = `/api/v1/apps/${app.id}`
  const payload = Buffer.from(
    'UEsDBBQAAAAIAKMRW13MJrglFQAAABMAAAARAAAAQXNzZXRNYW5pZmVzdC5iaW5LLC5OLdEtSS0u0c1NzMtMSy0uAQBQSwMEFAAAAAgAoxFbXcDaelMyAAAAOgAAABEAAABhc3NldHMvdXBkYXRlLnR4dCstSEksSU1RSCwuTi1RsClOLsosKLErz8xLyS/XAwu6VqQml4KU2CqUFJWm2uhD1QAAUEsDBBQAAAAIAKMRW12J44QcSwAAAEoAAAAPAAAAYXNzZXRzL2xvZ28ucG5n6wzwc+flkuJiYGDg9fRwCWJgYGACYQ42BgaGom28KgwMDIKeLo4hFXOST8z9/cqew/iABEOaBffZ60L51xgYGBg8Xf1c1jklNAEAUEsDBBQAAAAIAKMRW13kyvA+JQAAACMAAAASAAAAYXNzZXRzL2NvbmZpZy5qc29uq1YqyUjNTVWyUlBKzkgsSs5PzFHSUVAqSy0qzszPU7JSMKoFAFBLAQIUABQAAAAIAKMRW13MJrglFQAAABMAAAARAAAAAAAAAAAAAACAAQAAAABBc3NldE1hbmlmZXN0LmJpblBLAQIUABQAAAAIAKMRW13A2npTMgAAADoAAAARAAAAAAAAAAAAAACAAUQAAABhc3NldHMvdXBkYXRlLnR4dFBLAQIUABQAAAAIAKMRW12J44QcSwAAAEoAAAAPAAAAAAAAAAAAAACAAaUAAABhc3NldHMvbG9nby5wbmdQSwECFAAUAAAACACjEVtd5MrwPiUAAAAjAAAAEgAAAAAAAAAAAAAAgAEdAQAAYXNzZXRzL2NvbmZpZy5qc29uUEsFBgAAAAAEAAQA+wAAAHIBAAAAAA==',
    'base64',
  )
  const hash = createHash('sha256').update(payload).digest('hex')
  const registration = await request.post(`${appPath}/patches/${patch.id}/asset-bundle`, {
    headers,
    data: { platform: 'android', sha256: hash, size: payload.length },
  })
  expect(registration.ok()).toBeTruthy()
  const bundle = await registration.json()
  const uploadPath = new URL(bundle.upload_url).pathname
  const damaged = Buffer.from(payload)
  damaged[0] = damaged[0]! ^ 1
  const rejected = await request.post(uploadPath, {
    data: damaged,
    headers: { 'Content-Type': 'application/zip' },
  })
  expect(rejected.status()).toBe(400)
  const uploaded = await request.post(uploadPath, {
    data: payload,
    headers: { 'Content-Type': 'application/zip' },
  })
  expect(uploaded.ok()).toBeTruthy()
  const repeated = await request.post(uploadPath, { data: payload })
  expect(repeated.ok()).toBeFalsy()

  async function check(channel = 'stable') {
    const response = await request.get('/api/v1/assets/check', {
      params: {
        app_id: app.id,
        release_version: release.version,
        channel,
        platform: 'android',
        patch_number: String(patch.number),
      },
    })
    expect(response.ok()).toBeTruthy()
    return response.json()
  }

  expect((await check()).asset_bundle_available).toBe(false)
  const channels = await (await request.get(`${appPath}/channels`, { headers })).json()
  const stable = channels.find((channel: { name: string }) => channel.name === 'stable')
  const promoted = await request.post(`${appPath}/patches/promote`, {
    headers,
    data: { patch_id: patch.id, channel_id: stable.id },
  })
  expect(promoted.ok()).toBeTruthy()
  const available = await check()
  expect(available.asset_bundle_available).toBe(true)
  expect(available.sha256).toBe(hash)
  expect(available.size).toBe(payload.length)
  const downloaded = await request.get(new URL(available.download_url).pathname)
  expect(downloaded.ok()).toBeTruthy()
  expect(await downloaded.body()).toEqual(payload)
  expect((await check('beta')).asset_bundle_available).toBe(false)

  const patchPath = `${appPath}/releases/${release.id}/patches/${patch.id}`
  const rolledBack = await request.post(`${patchPath}/rollback`, { headers })
  expect(rolledBack.ok()).toBeTruthy()
  expect((await check()).asset_bundle_available).toBe(false)
  const restored = await request.post(`${patchPath}/rollforward`, { headers })
  expect(restored.ok()).toBeTruthy()
  expect((await check()).asset_bundle_available).toBe(true)

  await signIn(page)
  await page.goto(`/apps/${app.id}`)
  await page
    .getByRole('navigation', { name: 'Main navigation' })
    .getByRole('link', { name: 'Patches', exact: true })
    .click()
  await expect(page.getByRole('columnheader', { name: 'Assets', exact: true })).toBeVisible()
  await page.getByRole('button', { name: `Inspect assets for patch ${patch.number}` }).click()
  const inspector = page.getByRole('dialog')
  await expect(
    inspector.getByRole('heading', { name: 'Assets for patch #1', exact: true }),
  ).toBeVisible()
  await expect(inspector.getByText('4 files', { exact: true })).toBeVisible()
  await inspector.getByRole('button', { name: /assets\/update.txt/ }).click()
  await expect(inspector.locator('pre')).toHaveText(
    'updated asset <script>window.assetExecuted = true</script>',
  )
  expect(await page.evaluate(() => Reflect.get(window, 'assetExecuted'))).toBeUndefined()
  await inspector.getByLabel('Search asset files').fill('logo')
  await expect(inspector.getByRole('button', { name: /assets\/update.txt/ })).toHaveCount(0)
  await inspector.getByRole('button', { name: /assets\/logo.png/ }).click()
  const image = inspector.getByRole('img', { name: 'Preview of assets/logo.png' })
  await expect(image).toBeVisible()
  await expect
    .poll(async () => image.evaluate((element) => (element as HTMLImageElement).naturalWidth))
    .toBe(2)
  await inspector.getByLabel('Search asset files').clear()
  await inspector.getByRole('button', { name: /AssetManifest.bin/ }).click()
  await expect(inspector.getByText('Preview is unavailable', { exact: false })).toBeVisible()
  const downloadPromise = page.waitForEvent('download')
  await inspector.getByRole('link', { name: 'Download ZIP' }).click()
  const zip = await downloadPromise
  expect(zip.suggestedFilename()).toBe(`asset-bundle-${bundle.id}.zip`)
  expect(await zip.failure()).toBeNull()
  await inspector.getByRole('button', { name: /assets\/config.json/ }).click()
  await expect(inspector.locator('pre')).toContainText('charcoal')
  await page.screenshot({ path: 'test-results/asset-inspector-desktop.png', fullPage: true })
  await page.setViewportSize({ width: 390, height: 844 })
  await expect(inspector).toBeVisible()
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
  ).toBeTruthy()
  await page.screenshot({ path: 'test-results/asset-inspector-mobile.png', fullPage: true })
  await inspector.getByRole('button', { name: 'Close', exact: true }).click()
  await expect(inspector).not.toBeVisible()
})
