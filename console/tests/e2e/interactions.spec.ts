import { test, expect, type Page } from '@playwright/test'

const password = process.env.E2E_PASSWORD
const token = process.env.E2E_ADMIN_TOKEN

if (!password || !token) {
  throw new Error('Set isolated E2E_PASSWORD and E2E_ADMIN_TOKEN values for browser tests')
}

async function signIn(page: Page) {
  await page.goto('/login')
  const submit = page.getByRole('button', { name: 'Sign in', exact: true })
  await expect(submit).toHaveCSS('cursor', 'pointer')
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  const responsePromise = page.waitForResponse(
    (response) => new URL(response.url()).pathname === '/auth/login',
  )
  await submit.click()
  const response = await responsePromise
  if (response.status() === 429) {
    // The full suite intentionally exercises more logins than one production rate window.
    // Respect the server's cooldown instead of weakening the production limit for tests.
    await page.waitForTimeout(Math.min(60, Number(response.headers()['retry-after']) || 60) * 1000)
    await page.getByLabel('Password', { exact: true }).fill(password!)
    await submit.click()
  }
  await expect(page.getByRole('heading', { name: 'Overview', exact: true })).toBeVisible()
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const violations: string[] = []
    Object.defineProperty(window, '__interactionCspViolations', { value: violations })
    document.addEventListener('securitypolicyviolation', (event) => {
      violations.push(`${event.violatedDirective}: ${event.blockedURI}`)
    })
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: {
        writeText: async () => {
          if (Reflect.get(window, '__rejectClipboard')) {
            throw new Error('Clipboard access denied for this test')
          }
        },
      },
    })
  })
})

test.afterEach(async ({ page }) => {
  expect(
    await page.evaluate(() => Reflect.get(window, '__interactionCspViolations') ?? []),
  ).toEqual([])
})

test('interactive controls show pointer cursors and success toasts stay in the viewport', async ({
  page,
  request,
}) => {
  test.setTimeout(90_000)
  const applicationName = `Interaction fixture ${Date.now()}`
  const application = await request.post('/api/v1/apps', {
    headers: { Authorization: `Bearer ${token}` },
    data: { display_name: applicationName },
  })
  expect(application.ok()).toBeTruthy()
  await signIn(page)

  const picker = page.getByRole('combobox', { name: 'Selected application' })
  await expect(picker).toHaveCSS('cursor', 'pointer')
  await picker.click()
  const option = page.getByRole('option', { name: applicationName, exact: true })
  await expect(option).toHaveCSS('cursor', 'pointer')
  await option.click()

  const settings = page
    .getByRole('navigation', { name: 'Main navigation' })
    .getByRole('link', { name: 'Settings', exact: true })
  await expect(settings).toHaveCSS('cursor', 'pointer')
  await settings.click()
  const copy = page.getByRole('button', { name: 'Copy API origin' })
  await expect(copy).toHaveCSS('cursor', 'pointer')
  await expect(page.getByTestId('access-management')).toHaveAttribute('aria-busy', 'false')
  const heightBefore = await page.evaluate(() => document.documentElement.scrollHeight)
  await copy.click()

  const toaster = page.locator('[data-sonner-toaster]')
  const toast = page.locator('[data-sonner-toast]').filter({ hasText: 'API origin copied' })
  await expect(toaster).toHaveCSS('position', 'fixed')
  await expect(toast).toBeVisible()
  await expect(toast).toHaveCSS('opacity', '1')
  await expect(toast).toBeInViewport({ ratio: 1 })
  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(heightBefore)
  await page.screenshot({ path: 'test-results/toast-desktop.png', fullPage: true })
  await toast.getByRole('button', { name: 'Close toast' }).click()
  await expect(toast).not.toBeVisible()
})

test('error toasts are visible on mobile without increasing document height', async ({ page }) => {
  test.setTimeout(90_000)
  await page.setViewportSize({ width: 390, height: 844 })
  await signIn(page)
  await page.goto('/settings')
  await expect(page.getByRole('heading', { name: 'Settings', exact: true })).toBeVisible()
  await page.evaluate(() => Reflect.set(window, '__rejectClipboard', true))
  await expect(page.getByTestId('access-management')).toHaveAttribute('aria-busy', 'false')
  const heightBefore = await page.evaluate(() => document.documentElement.scrollHeight)
  await page.getByRole('button', { name: 'Copy API origin' }).click()

  const toast = page.locator('[data-sonner-toast]').filter({ hasText: 'Clipboard unavailable.' })
  await expect(toast).toBeVisible()
  await expect(toast).toHaveCSS('opacity', '1')
  await expect(toast).toBeInViewport({ ratio: 1 })
  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(heightBefore)
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
  ).toBeTruthy()
  await page.screenshot({ path: 'test-results/toast-mobile.png', fullPage: true })
})
