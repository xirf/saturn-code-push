import { describe, expect, it } from 'vitest'
import { summarizeEvents } from '../../app/features/telemetry/lib/metrics'
import type { DeviceEvent } from '../../app/features/telemetry/types/event'

function event(
  appId: string,
  receivedAt: string,
  type: string,
  clientId = 'installation',
): DeviceEvent {
  return { received_at: receivedAt, event: { app_id: appId, type, client_id: clientId } }
}

describe('retained-window analytics', () => {
  it('scopes reports by app and UTC calendar days, including the entire first day', () => {
    const records = [
      event('app', '2026-10-21T00:00:00Z', '__patch_download__'),
      event('app', '2026-10-27T23:59:59Z', '__patch_install__'),
      event('app', '2026-10-20T23:59:59Z', '__patch_download__'),
      event('other', '2026-10-25T12:00:00Z', '__patch_download__'),
      event('app', '2026-10-28T00:00:00Z', '__patch_download__'),
    ]
    const result = summarizeEvents(records, 'app', 7, new Date('2026-10-27T10:00:00Z'))
    expect(result.downloads).toBe(1)
    expect(result.launches).toBe(1)
    expect(result.chart).toHaveLength(7)
    expect(result.chart[0]).toEqual({ day: '2026-10-21', downloads: 1, launches: 0 })
    expect(result.reporting).toBe(1)
  })

  it('counts duplicate reports but deduplicates nonempty installation identifiers', () => {
    const records = [
      event('app', '2026-10-27T01:00:00Z', '__patch_download__', 'same'),
      event('app', '2026-10-27T01:00:00Z', '__patch_download__', 'same'),
      event('app', '2026-10-27T02:00:00Z', '__patch_install_failure__', ''),
      event('app', '2026-10-27T03:00:00Z', '__patch_update_failure__', 'different'),
    ]
    const result = summarizeEvents(records, 'app', 7, new Date('2026-10-27T10:00:00Z'))
    expect(result.downloads).toBe(2)
    expect(result.reporting).toBe(2)
    expect(result.failures).toBe(2)
  })
})
