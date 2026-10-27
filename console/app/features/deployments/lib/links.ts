export interface Deployment {
  id: number
  release_id: number
  key: string
  repository: string
  commit: string
  source_kind: string
  source_ref: string
  destination: string
  status: string
  build_number: string
  workflow_run: number | null
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
  return items.find(
    (item) =>
      item.id < current.id &&
      item.destination === current.destination &&
      item.repository === current.repository &&
      item.status === 'published',
  )
}
