import { useMutation, useQueryCache } from '@pinia/colada'
import { toast } from 'vue-sonner'
import { request } from '~/lib/api/client'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import type { Patch, PatchAction } from '../types/patch'

interface PatchActionInput {
  appId: string
  patch: Patch
  action: PatchAction
  channelId?: number
  generation: number
}

export function usePatchAction() {
  const cache = useQueryCache()
  const scope = useSessionScope()

  return useMutation({
    key: (input) => ['private', input.generation, 'patch-action'],
    mutation: async (input: PatchActionInput) => {
      const appPath = `/api/v1/apps/${encodeURIComponent(input.appId)}`
      if (input.action === 'promote') {
        if (!input.channelId) throw new Error('Select a delivery channel')
        await request(`${appPath}/patches/promote`, {
          method: 'POST',
          body: { patch_id: input.patch.id, channel_id: input.channelId },
        })
      } else {
        const operation = input.action === 'rollback' ? 'rollback' : 'rollforward'
        await request(
          `${appPath}/releases/${input.patch.release_id}/patches/${input.patch.id}/${operation}`,
          {
            method: 'POST',
            body: {},
          },
        )
      }
      return input
    },
    onSuccess: async (input) => {
      if (input.generation !== scope.generation) return
      await cache.invalidateQueries({ key: ['private', input.generation] })
      toast.success(
        input.action === 'promote'
          ? 'Patch promoted'
          : input.action === 'rollback'
            ? 'Patch rolled back'
            : 'Patch restored',
        { description: 'Devices receive the updated state on their next check.' },
      )
    },
  })
}
