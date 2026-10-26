import { test, expect } from '@playwright/test'

test('public guides support search, headings, code copy, and direct reloads', async ({ page }) => {
  const privateRequests: string[] = []
  const errors: string[] = []
  page.on('request', (request) => {
    if (/\/(auth|admin)\//.test(new URL(request.url()).pathname))
      privateRequests.push(request.url())
  })
  page.on('pageerror', (error) => errors.push(error.message))
  await page.addInitScript(() => {
    const violations: string[] = []
    Object.defineProperty(window, '__docsCspViolations', { value: violations })
    document.addEventListener('securitypolicyviolation', (event) => {
      violations.push(`${event.violatedDirective}: ${event.blockedURI}`)
    })
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: async (value: string) => Reflect.set(window, '__copiedCode', value) },
    })
  })

  await page.goto('/docs')
  await expect(page.getByRole('heading', { name: 'Documentation', exact: true })).toBeVisible()
  const navigation = page.getByRole('navigation', { name: 'Documentation', exact: true })
  await page.getByLabel('Search documentation').fill('no-matching-guide-123')
  await expect(navigation.getByText('No guides match your search.')).toBeVisible()
  await page.getByLabel('Search documentation').fill('asset')
  await navigation.getByRole('link', { name: 'Asset updates', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Publish and inspect asset updates', exact: true }),
  ).toBeVisible()
  await page
    .getByRole('navigation', { name: 'On this page' })
    .getByRole('link', { name: 'Publish an asset update', exact: true })
    .click()
  await expect(page).toHaveURL(/#publish-an-asset-update$/)
  await page.getByRole('button', { name: 'Copy code', exact: true }).first().click()
  await expect(page.locator('[data-sonner-toast]').filter({ hasText: 'Code copied' })).toBeVisible()
  expect(await page.evaluate(() => Reflect.get(window, '__copiedCode'))).toContain(
    '--allow-asset-diffs',
  )
  await page.reload()
  await expect(
    page.getByRole('heading', { name: 'Publish and inspect asset updates', exact: true }),
  ).toBeVisible()
  expect(privateRequests).toEqual([])
  expect(errors).toEqual([])
  expect(await page.evaluate(() => Reflect.get(window, '__docsCspViolations'))).toEqual([])
  await page.screenshot({ path: 'test-results/documentation-desktop.png' })
})

test('public documentation works on mobile and links back to sign-in', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/docs/shorebird')
  await expect(
    page.getByRole('heading', { name: 'Using Saturn with Shorebird', exact: true }),
  ).toBeVisible()
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
  ).toBeTruthy()
  await page.getByRole('button', { name: 'Open documentation navigation' }).click()
  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  await dialog.getByRole('link', { name: 'License', exact: true }).click()
  await expect(dialog).not.toBeVisible()
  await expect(page.getByRole('heading', { name: 'Apache License 2.0', exact: true })).toBeVisible()
  await page.screenshot({ path: 'test-results/documentation-mobile.png' })
  await page.getByRole('link', { name: 'Open console', exact: true }).click()
  await expect(page).toHaveURL(/\/login$/)
  await page.getByRole('link', { name: 'Documentation', exact: true }).click()
  await expect(page).toHaveURL(/\/docs$/)
})
