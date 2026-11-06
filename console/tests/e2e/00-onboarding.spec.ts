import { test, expect } from '@playwright/test'
const setupToken = process.env.E2E_SETUP_TOKEN
const password = process.env.E2E_PASSWORD

test('first run creates the administrator in the browser and closes setup', async ({
  page,
  request,
}) => {
  test.skip(!setupToken || !password, 'Requires a fresh server with E2E_SETUP_TOKEN')
  await page.goto('/')
  await expect(page).toHaveURL(/\/setup$/)
  await expect(page.getByRole('heading', { name: 'Set up your workspace' })).toBeVisible()
  await page.getByLabel('Setup token', { exact: true }).fill(setupToken!)
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByLabel('Confirm password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Create administrator', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible()
  const session = await page.request.get('/auth/session')
  expect((await session.json()).role).toBe('admin')
  expect(
    await page.evaluate(() => JSON.stringify({ ...localStorage, ...sessionStorage })),
  ).not.toContain(setupToken!)
  const closed = await request.post('/auth/setup', {
    data: { setup_token: setupToken, username: 'another-admin', password },
    headers: { Origin: new URL(page.url()).origin },
  })
  expect(closed.status()).toBe(409)
  await page.getByRole('button', { name: 'Sign out', exact: true }).click()
  await expect(page).toHaveURL(/\/login$/)
  await page.goto('/setup')
  await expect(page).toHaveURL(/\/login$/)
})
