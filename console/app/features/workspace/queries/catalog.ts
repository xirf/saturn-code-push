import { defineQuery, useQuery } from '@pinia/colada'
import { watch } from 'vue'
import { ApiError, request } from '~/lib/api/client'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { catalogSchema } from '../types/catalog'

export const useCatalogQuery = defineQuery(() => {
  const identity = useIdentity()
  const scope = useSessionScope()
  const query = useQuery({
    key: () => ['private', scope.generation, 'catalog'],
    enabled: () => Boolean(identity.session.data.value),
    staleTime: 15_000,
    query: async ({ signal }) => catalogSchema.parse(await request('/admin/state', { signal })),
  })

  watch(query.error, (error) => {
    if (error instanceof ApiError && error.status === 401) void identity.expire()
  })
  return query
})
