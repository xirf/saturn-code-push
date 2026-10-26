export function formatDate(value: string): string {
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? 'Unknown date'
    : new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(date)
}

export function formatBytes(size: number): string {
  return size >= 1024 * 1024
    ? `${(size / (1024 * 1024)).toFixed(1)} MiB`
    : `${Math.ceil(size / 1024)} KiB`
}

export function shortId(value: string): string {
  return value.slice(0, 8)
}
