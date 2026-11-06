import { test, expect } from '@playwright/test'
const password = process.env.E2E_PASSWORD
const token = process.env.E2E_ADMIN_TOKEN
if (!password || !token) throw new Error('Set isolated E2E_PASSWORD and E2E_ADMIN_TOKEN values')

test('administrators manage accounts and keys; viewers cannot publish', async ({
  page,
  request,
}) => {
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  const appResult = await request.post('/api/v1/apps', {
    headers: { Authorization: `Bearer ${token}` },
    data: { display_name: 'Access browser fixture' },
  })
  expect(appResult.ok()).toBeTruthy()
  const app = await appResult.json()
  await page.goto('/login')
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible()
  await page.goto('/settings')
  await page.getByRole('link', { name: 'Accounts Manage users' }).click()
  await page.getByRole('button', { name: 'New account', exact: true }).click()
  const dialog = page.getByRole('dialog')
  const username = `viewer-${Date.now()}`
  await dialog.getByLabel('Username', { exact: true }).fill(username)
  await dialog.getByLabel('Password', { exact: true }).fill(password!)
  await dialog.getByRole('button', { name: 'Save account' }).click()
  await expect(dialog).not.toBeVisible()
  await expect(page.getByRole('row').filter({ hasText: username })).toContainText('viewer')
  // Keys live inside their application; the dialog is already scoped to it.
  await page.goto(`/apps/${app.id}/settings/api-keys`)
  await page.getByRole('button', { name: 'New API key', exact: true }).click()
  await page.getByLabel('Key name', { exact: true }).fill('Actions browser key')
  await expect(page.getByLabel('Application', { exact: true })).toHaveCount(0)
  await page.getByRole('button', { name: 'Create API key', exact: true }).click()
  const secret = page.getByRole('textbox', { name: 'New API key', exact: true })
  await expect(secret).toBeVisible()
  expect(await secret.inputValue()).toMatch(/^sb_api_/)
  expect(
    await page.evaluate(() => JSON.stringify({ ...localStorage, ...sessionStorage })),
  ).not.toContain(await secret.inputValue())
  await page.getByRole('button', { name: 'I have saved the key' }).click()
  await page.getByRole('button', { name: 'Revoke key Actions browser key', exact: true }).click()
  await expect(page.getByRole('row').filter({ hasText: 'Actions browser key' })).toContainText(
    'Revoked',
  )
  await page.goto('/settings/management/audit')
  await expect(page).toHaveURL(/\/settings\/management\/audit$/)
  await expect(page.getByRole('heading', { name: 'Audit history', exact: true })).toBeVisible()
  await expect(page.getByRole('table')).toContainText('/admin/api-keys')
  await page.getByRole('link', { name: 'Back to settings' }).click()
  await page.getByRole('button', { name: 'Sign out', exact: true }).first().click()
  await expect(page).toHaveURL(/\/login$/)
  await page.getByLabel('Username', { exact: true }).fill(username)
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible()
  await page.goto('/')
  await expect(page.getByRole('link', { name: 'New application', exact: true })).toHaveCount(0)
  await page.goto('/settings')
  await expect(page.getByRole('button', { name: 'New account', exact: true })).toHaveCount(0)
  await expect(page.getByRole('link', { name: 'Accounts Manage users' })).toHaveCount(0)
  await expect(page.getByRole('link', { name: 'API keys Application' })).toHaveCount(0)
  await page.getByRole('link', { name: 'Account & security Your profile' }).click()
  await expect(page.getByRole('button', { name: 'Change password and sign out' })).toBeVisible()
  await page.goto('/settings/management/audit')
  await expect(page.getByRole('alert')).toContainText('Only administrators')
  expect(errors).toEqual([])
})

test('settings provides focused destinations on desktop and mobile', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 })
  await page.goto('/login')
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible()
  await page.goto('/settings')
  await expect(page.getByRole('heading', { name: 'Management', exact: true })).toBeVisible()
  await expect(page.locator('form')).toHaveCount(0)
  await expect(page.getByRole('navigation', { name: 'Settings sections' })).toHaveCount(0)
  await expect(
    page
      .getByRole('navigation', { name: 'Main navigation' })
      .getByRole('link', { name: 'Audit history' }),
  ).toHaveCount(0)
  await page.screenshot({
    path: 'test-results/settings-index-desktop.png',
    fullPage: true,
    animations: 'disabled',
  })
  await page.setViewportSize({ width: 390, height: 844 })
  await page.screenshot({
    path: 'test-results/settings-index-mobile.png',
    fullPage: true,
    animations: 'disabled',
  })
  await page.getByRole('link', { name: 'API keys Manage application' }).click()
  await expect(page.getByRole('heading', { name: 'API keys', exact: true })).toBeVisible()
  await expect(page.locator('form')).toHaveCount(0)
  await page.getByRole('button', { name: 'New API key', exact: true }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.screenshot({
    path: 'test-results/settings-key-mobile.png',
    fullPage: true,
    animations: 'disabled',
  })
  await page.getByRole('button', { name: 'Close', exact: true }).click()
  await page.getByRole('link', { name: 'Back to settings', exact: true }).click()
  await page.setViewportSize({ width: 320, height: 740 })
  for (const path of [
    '/settings',
    '/settings/account',
    '/settings/management/accounts',
    '/settings/management/audit',
    '/settings/server/connection',
    '/settings/server/storage',
    '/apps/missing/settings',
  ]) {
    await page.goto(path)
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible()
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      path,
    ).toBeTruthy()
  }
})

test('audit history loads on its own page and supports older and newer pages', async ({ page }) => {
  const cursors: (string | null)[] = []
  await page.route('**/admin/audit*', async (route) => {
    const before = new URL(route.request().url()).searchParams.get('before')
    cursors.push(before)
    const events = Array.from({ length: before ? 1 : 100 }, (_, index) => ({
      id: before ? 1 : 200 - index,
      actor_id: 'admin-fixture',
      actor_name: before ? 'Older actor' : 'Recent actor',
      actor_kind: 'account',
      action: 'POST',
      resource: '/admin/api-keys',
      app_id: null,
      occurred_at: '2026-10-27T10:30:00Z',
    }))
    await route.fulfill({ json: { events } })
  })
  await page.goto('/login')
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible()
  await page.goto('/settings')
  await expect(page.getByRole('link', { name: 'Audit history Review changes' })).toBeVisible()
  expect(cursors).toEqual([])
  await page.getByRole('link', { name: 'Audit history Review changes' }).click()
  const pagination = page.getByRole('navigation', { name: 'Audit history pages' })
  await expect(pagination).toContainText('Page 1 · 100 events')
  await expect(page.getByRole('button', { name: 'Newer events' })).toBeDisabled()
  await page.getByRole('button', { name: 'Older events' }).click()
  await expect(pagination).toContainText('Page 2 · 1 event')
  await expect(page.getByRole('table')).toContainText('Older actor')
  await expect(page.getByRole('button', { name: 'Older events' })).toBeDisabled()
  expect(cursors).toEqual([null, '101'])
  await page.getByRole('button', { name: 'Newer events' }).click()
  await expect(pagination).toContainText('Page 1 · 100 events')
  await expect(page.getByRole('table')).toContainText('Recent actor')
})
