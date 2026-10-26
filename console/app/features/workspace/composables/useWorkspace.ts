import { computed } from 'vue'
import { usePreferencesStore } from '~/stores/preferences'
import { useCatalogQuery } from '../queries/catalog'

export function useWorkspace() {
  const query = useCatalogQuery()
  const preferences = usePreferencesStore()
  const applications = computed(() => query.data.value?.apps ?? [])
  const application = computed(
    () =>
      applications.value.find((app) => app.id === preferences.selectedAppId) ??
      applications.value[0],
  )
  const appId = computed(() => application.value?.id ?? '')
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

  return {
    query,
    applications,
    application,
    appId,
    releases,
    patches,
    channels,
    artifacts,
    bundles,
    events,
  }
}
