import { computed, type Ref } from 'vue'
import { formatBytes } from '~/lib/format'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import type { Patch } from '../types/patch'

export function usePatchRows(patches: Ref<Patch[]>) {
  const workspace = useWorkspace()

  return computed(() =>
    patches.value.map((patch) => {
      const release = workspace.releases.value.find((release) => release.id === patch.release_id)
      const channels = patch.channel_ids.map((id) => {
        const name =
          workspace.channels.value.find((channel) => channel.id === id)?.name ?? 'Unknown'
        const percentage = patch.rollout_percentages[String(id)] ?? 100
        return percentage === 100
          ? name
          : `${name} · ${percentage === 0 ? 'Paused' : `${percentage}%`}`
      })
      const artifacts = workspace.artifacts.value
        .filter((artifact) => artifact.patch_id === patch.id)
        .map((artifact) => {
          const pending = artifact.uploaded ? '' : ' · pending'
          return `${artifact.arch} · ${formatBytes(artifact.size)}${pending}`
        })
        .join(', ')

      return {
        patch,
        channels,
        releaseVersion: release?.version ?? 'Unknown',
        artifacts: artifacts || 'No artifacts',
        status: patch.is_rolled_back
          ? 'Rolled back'
          : channels.length
            ? 'Published'
            : 'Unpublished',
      }
    }),
  )
}
