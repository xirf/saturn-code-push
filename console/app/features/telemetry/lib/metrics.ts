import type { DeviceEvent } from '../types/event'

export function summarizeEvents(
  events: DeviceEvent[],
  appId: string,
  days: number,
  now = new Date(),
) {
  const dayCount = Math.min(30, Math.max(1, Math.trunc(days)))
  const end = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() + 1)
  const start = end - dayCount * 86_400_000
  const period = events.filter((record) => {
    const timestamp = Date.parse(record.received_at)
    return record.event.app_id === appId && timestamp >= start && timestamp < end
  })
  const chart = Array.from({ length: dayCount }, (_, index) => {
    const day = new Date(start + index * 86_400_000).toISOString().slice(0, 10)
    const records = period.filter((record) => record.received_at.slice(0, 10) === day)
    return {
      day,
      downloads: records.filter((record) => record.event.type === '__patch_download__').length,
      launches: records.filter((record) => record.event.type === '__patch_install__').length,
    }
  })
  return {
    period,
    chart,
    downloads: period.filter((record) => record.event.type === '__patch_download__').length,
    launches: period.filter((record) => record.event.type === '__patch_install__').length,
    failures: period.filter((record) =>
      ['__patch_install_failure__', '__patch_update_failure__'].includes(record.event.type),
    ).length,
    reporting: new Set(period.map((record) => record.event.client_id).filter(Boolean)).size,
  }
}

export function eventLabel(type: string): string {
  return (
    (
      {
        __patch_download__: 'Patch downloaded',
        __patch_install__: 'Patch launched',
        __patch_install_failure__: 'Launch failed',
        __patch_update_failure__: 'Update failed',
      } as Record<string, string>
    )[type] ?? 'Device report'
  )
}
