import { computed } from 'vue'
import { usePreferencesStore } from '~/stores/preferences'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { summarizeEvents } from '../lib/metrics'

export function useTelemetry() {
  const workspace = useWorkspace()
  const preferences = usePreferencesStore()
  const summary = computed(() =>
    summarizeEvents(workspace.events.value, workspace.appId.value, preferences.periodDays),
  )
  return { summary, preferences }
}
