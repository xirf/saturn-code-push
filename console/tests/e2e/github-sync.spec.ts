import { test, expect } from '@playwright/test'
const token = process.env.E2E_ADMIN_TOKEN
const password = process.env.E2E_PASSWORD
if (!token || !password) throw new Error('Use isolated E2E credentials')

test('GitHub connection controls and patch reports keep CI separate from deployment status', async ({
  page,
  request,
}) => {
  const headers = { Authorization: `Bearer ${token}` }
  const app = await (
    await request.post('/api/v1/apps', {
      headers,
      data: { display_name: `GitHub fixture ${Date.now()}` },
    })
  ).json()
  const { release } = await (
    await request.post(`/api/v1/apps/${app.id}/releases`, {
      headers,
      data: { version: '1.0.0+1', flutter_revision: 'github-fixture' },
    })
  ).json()
  const patch = await (
    await request.post(`/api/v1/apps/${app.id}/patches`, {
      headers,
      data: { release_id: release.id },
    })
  ).json()
  let connected = false
  let connectionBody: unknown
  await page.route('**/admin/integrations/github', async (route) => {
    if (route.request().method() === 'POST') {
      connectionBody = route.request().postDataJSON()
      connected = true
      await route.fulfill({ json: { connected: true } })
    } else {
      await route.fulfill({
        json: {
          configured: true,
          installation_url: 'https://github.com/apps/example/installations/new',
          webhook_url: 'https://example.com/integrations/github/webhook',
          connections: connected
            ? [
                {
                  app_id: app.id,
                  repository: 'owner/repo',
                  installation_id: 7,
                  connected_at: '2026-10-27T00:00:00Z',
                },
              ]
            : [],
        },
      })
    }
  })
  await page.goto('/login')
  await page.getByLabel('Username', { exact: true }).fill('admin')
  await page.getByLabel('Password', { exact: true }).fill(password!)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Overview', exact: true })).toBeVisible()
  await page.goto('/settings/integrations/github')
  await page.getByRole('button', { name: 'Connect a repository', exact: true }).click()
  await page.getByRole('combobox', { name: 'GitHub application' }).click()
  await page.getByRole('option', { name: app.display_name }).click()
  await page.getByLabel('GitHub repository', { exact: true }).fill('owner/repo')
  await page.getByLabel('Installation ID', { exact: true }).fill('7')
  await page.getByRole('button', { name: 'Connect repository', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Disconnect owner/repo' })).toBeVisible()
  expect(connectionBody).toEqual({ app_id: app.id, repository: 'owner/repo', installation_id: 7 })
  await page.goto(`/releases/${release.id}`)
  await page.getByText('Link a deployment manually', { exact: true }).click()
  await page.getByRole('combobox', { name: 'Deployment target' }).click()
  await page.getByRole('option', { name: `Patch #${patch.number}` }).click()
  await page.getByLabel('GitHub repository (owner/repo)', { exact: true }).fill('owner/repo')
  await page.getByLabel('Full commit SHA', { exact: true }).fill('a'.repeat(40))
  await page.getByLabel('Branch, tag or release tag', { exact: true }).fill('main')
  await page.getByLabel('Destination', { exact: true }).fill('ota/stable')
  await page.getByLabel('Actions run ID (optional)', { exact: true }).fill('42')
  await page
    .getByLabel('Deployment key (optional; reuse to update)', { exact: true })
    .fill('github-browser-fixture')
  await page.getByRole('button', { name: 'Save deployment', exact: true }).click()
  await expect(page.getByText(/1\.0\.0\+1 · Patch #1/)).toBeVisible()
  await page.route(`**/api/v1/apps/${app.id}/deployments`, async (route) => {
    const response = await route.fetch()
    const items = await response.json()
    for (const item of items)
      item.workflow = {
        run_id: 42,
        run_attempt: 2,
        status: 'completed',
        conclusion: 'failure',
        checked_at: '2026-10-27T00:00:00Z',
        sync_error: null,
      }
    await route.fulfill({ json: items })
  })
  await page.reload()
  await expect(page.getByText(/CI: completed.*failure.*Attempt 2/)).toBeVisible()
  await expect(page.getByLabel('Reported deployment status', { exact: true })).toHaveValue(
    'uploaded',
  )
  const records = await (
    await request.get(`/api/v1/apps/${app.id}/deployments`, { headers })
  ).json()
  expect(records[0].patch_id).toBe(patch.id)
  expect(records[0].status).toBe('uploaded')
})
