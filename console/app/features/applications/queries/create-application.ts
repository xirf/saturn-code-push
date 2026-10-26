import { useMutation, useQueryCache } from '@pinia/colada'
import { toast } from 'vue-sonner'
import { request } from '~/lib/api/client'
import { usePreferencesStore } from '~/stores/preferences'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { applicationSchema } from '../types/application'

export function useCreateApplication() {
  const cache = useQueryCache()
  const scope = useSessionScope()
  const preferences = usePreferencesStore()

  return useMutation({
    key: (input) => ['private', input.generation, 'create-application'],
    mutation: async (input: { display_name: string; generation: number }) => ({
      app: applicationSchema.parse(
        await request('/api/v1/apps', {
          method: 'POST',
          body: { display_name: input.display_name, organization_id: 1 },
        }),
      ),
      generation: input.generation,
    }),
    onSuccess: async ({ app, generation }) => {
      if (generation !== scope.generation) return
      preferences.selectedAppId = app.id
      await cache.invalidateQueries({ key: ['private', generation] })
      toast.success('Application created', { description: 'Ready for its first release.' })
    },
  })
}
