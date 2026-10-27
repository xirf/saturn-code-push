import { useQuery, useQueryCache } from '@pinia/colada'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { request } from '~/lib/api/client'
import { accountsSchema, keysSchema, auditSchema, newKeySchema } from '../types/access'

export function useAccess() {
  const identity = useIdentity()
  const scope = useSessionScope()
  const cache = useQueryCache()
  const admin = computed(() => identity.session.data.value?.role === 'admin')
  const before = ref<number | null>(null)
  const accounts = useQuery({
    key: () => ['private', scope.generation, 'accounts'],
    enabled: () => admin.value,
    query: async ({ signal }) => accountsSchema.parse(await request('/admin/accounts', { signal })),
  })
  const keys = useQuery({
    key: () => ['private', scope.generation, 'api-keys'],
    enabled: () => admin.value,
    query: async ({ signal }) => keysSchema.parse(await request('/admin/api-keys', { signal })),
  })
  const audit = useQuery({
    key: () => ['private', scope.generation, 'audit', before.value],
    enabled: () => admin.value,
    query: async ({ signal }) =>
      auditSchema.parse(
        await request(`/admin/audit${before.value ? `?before=${before.value}` : ''}`, { signal }),
      ),
  })
  async function change(path: string, body: unknown, method: 'POST' | 'PATCH' = 'POST') {
    const generation = scope.generation
    const result = await request(path, { method, body })
    if (scope.generation !== generation) throw new Error('Your session changed. Sign in again.')
    await cache.invalidateQueries({ key: ['private', generation] })
    return result
  }
  return {
    admin,
    accounts,
    keys,
    audit,
    before,
    createAccount: (body: { username: string; password: string; role: string }) =>
      change('/admin/accounts', body),
    updateAccount: (id: string, body: { role: string; disabled: boolean; password?: string }) =>
      change(`/admin/accounts/${encodeURIComponent(id)}`, body, 'PATCH'),
    createKey: async (body: {
      name: string
      app_id: string
      permission: string
      expires_in_days: number
    }) => newKeySchema.parse(await change('/admin/api-keys', body)),
    revokeKey: (id: string) => change(`/admin/api-keys/${encodeURIComponent(id)}/revoke`, {}),
    changePassword: async (body: { current_password: string; password: string }) => {
      await request('/auth/password', { method: 'POST', body })
      await identity.expire()
    },
  }
}
