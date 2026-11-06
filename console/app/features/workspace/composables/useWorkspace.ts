import { computed } from 'vue'
import { useCatalogQuery } from '../queries/catalog'

/**
 * Projections of the catalog for the application named in the URL (`/apps/:appId/...`).
 * Outside an application route every list is empty and `application` is undefined.
 */
export function useWorkspace() {
  const query = useCatalogQuery()
  const route = useRoute()
  const applications = computed(() => query.data.value?.apps ?? [])
  const routeAppId = computed(() => {
    const value = route.params.appId
    return typeof value === 'string' ? value : ''
  })
  const application = computed(() => applications.value.find((app) => app.id === routeAppId.value))
  const appId = computed(() => application.value?.id ?? '')
  const basePath = computed(() => `/apps/${encodeURIComponent(routeAppId.value)}`)
  const releases = computed(() =>
    (query.data.value?.releases ?? [])
      .filter((release) => release.app_id === appId.value)
      .slice()
      .reverse(),
  )
  const releaseIds = computed(() => new Set(releases.value.map((release) => release.id)))
  const patches = computed(() =>
    (query.data.value?.patches ?? [])
      .filter((patch) => releaseIds.value.has(patch.release_id))
      .slice()
      .reverse(),
  )
  const channels = computed(() =>
    (query.data.value?.channels ?? []).filter((channel) => channel.app_id === appId.value),
  )
  const artifacts = computed(() =>
    (query.data.value?.artifacts ?? []).filter((artifact) => artifact.app_id === appId.value),
  )
  const bundles = computed(() =>
    (query.data.value?.asset_bundles ?? []).filter((bundle) => bundle.app_id === appId.value),
  )
  const events = computed(() =>
    (query.data.value?.events ?? []).filter((event) => event.event.app_id === appId.value),
  )

  function path(section = '') {
    return `${basePath.value}${section ? `/${section.replace(/^\//, '')}` : ''}`
  }

  return {
    query,
    applications,
    application,
    appId,
    basePath,
    path,
    releases,
    patches,
    channels,
    artifacts,
    bundles,
    events,
  }
}
