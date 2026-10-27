export interface Deployment {
  id: number
  release_id: number
  patch_id?: number | null
  workflow?: {
    run_id: number
    run_attempt: number
    status: string | null
    conclusion: string | null
    checked_at: string
    sync_error: string | null
  } | null
  key: string
  repository: string
  commit: string
  source_kind: string
  source_ref: string
  destination: string
  status: string
  build_number: string
  workflow_run: number | null
  workflow_attempt?: number | null
  created_at: string
  updated_at: string
  reported_by?: string | null
  updated_by?: string | null
}

export function githubUrl(item: Deployment) {
  return `https://github.com/${item.repository.split('/').map(encodeURIComponent).join('/')}`
}

export function comparisonUrl(base: Deployment | undefined, current: Deployment) {
  if (!base || base.id === current.id || base.repository !== current.repository) return undefined
  return `${githubUrl(current)}/compare/${encodeURIComponent(base.commit)}...${encodeURIComponent(current.commit)}`
}

export function previousPublished(items: Deployment[], current: Deployment) {
  const published = items.filter(
    (item) =>
      item.id < current.id &&
      item.destination === current.destination &&
      item.repository === current.repository &&
      item.status === 'published',
  )
  if (current.patch_id != null) {
    return (
      published.find((item) => item.release_id === current.release_id && item.patch_id != null) ??
      published.find((item) => item.release_id === current.release_id && item.patch_id == null)
    )
  }
  return published.find((item) => item.patch_id == null)
}
