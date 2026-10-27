import { describe, expect, it } from 'vitest'
import { catalogSchema } from '../../app/features/workspace/types/catalog'

describe('device event validation', () => {
  it('accepts device reports with a null, absent, or textual message', () => {
    const event = {
      app_id: 'app',
      type: '__patch_download__',
      release_version: '1.0.0+1',
      patch_number: 1,
      client_id: 'device',
      platform: 'android',
      arch: 'aarch64',
      timestamp: 123,
    }
    const record = (message: unknown) => ({
      received_at: '2026-10-27T00:00:00Z',
      event: { ...event, ...(message === undefined ? {} : { message }) },
    })
    const catalog = {
      apps: [],
      channels: [],
      releases: [],
      patches: [],
      artifacts: [],
      events: [record(null), record(undefined), record('Download failed')],
    }
    expect(catalogSchema.safeParse(catalog).success).toBe(true)
    expect(catalogSchema.safeParse({ ...catalog, events: [record(42)] }).success).toBe(false)
  })
})
