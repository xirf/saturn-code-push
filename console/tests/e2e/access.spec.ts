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
  await expect(page.getByRole('heading', { name: 'Overview', exact: true })).toBeVisible()
  await page.goto('/settings')
  await page.getByRole('button', { name: 'New account', exact: true }).click()
  const dialog = page.getByRole('dialog')
  const username = `viewer-${Date.now()}`
  await dialog.getByLabel('Username', { exact: true }).fill(username)
  await dialog.getByLabel('Password', { exact: true }).fill(password!)
  await dialog.getByRole('button', { name: 'Save account' }).click()
  await expect(dialog).not.toBeVisible()
  await expect(page.getByRole('row').filter({ hasText: username })).toContainText('viewer')
  await page.getByLabel('Key name', { exact: true }).fill('Actions browser key')
  await page.getByLabel('Application', { exact: true }).selectOption(app.id)
  await page.getByRole('button', { name: 'Create API key', exact: true }).click()
  const secret = page.getByLabel('New API key', { exact: true })
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
  await page.getByRole('button', { name: 'Sign out', exact: true }).first().click()
  await expect(page).toHaveURL(/\/login$/)
  await page.getByLabel('Username', { exact: true }).fill(username)
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Overview', exact: true })).toBeVisible()
  await page.goto('/applications')
  await expect(page.getByRole('button', { name: 'New application', exact: true })).toHaveCount(0)
  await page.goto('/settings')
  await expect(page.getByRole('button', { name: 'New account', exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Change password and sign out' })).toBeVisible()
  expect(errors).toEqual([])
})
