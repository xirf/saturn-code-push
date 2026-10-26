import { computed, type Ref } from 'vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import type { Release } from '../types/release'

export function useReleaseRows(releases: Ref<Release[]>) {
  const workspace = useWorkspace()

  return computed(() =>
    releases.value.map((release) => {
      const active = Object.values(release.platforms).some(
        (platform) => platform.status === 'active',
      )
      const baselineCount = workspace.artifacts.value.filter((artifact) => {
        return artifact.release_id === release.id && artifact.patch_id === null && artifact.uploaded
      }).length

      return {
        release,
        active,
        baselineCount,
        platformNames: Object.keys(release.platforms).join(', ') || 'Not activated',
      }
    }),
  )
}
