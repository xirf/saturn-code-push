import { useMutation, useMutationCache, useQueryCache } from '@pinia/colada'
import { request } from '~/lib/api/client'
import { usePreferencesStore } from '~/stores/preferences'
import { useSessionScope } from '../stores/session-scope'
import { SESSION_KEY, useSessionQuery } from '../queries/session'
import { sessionSchema, type LoginInput } from '../types/session'

export function useIdentity() {
  const session = useSessionQuery()
  const cache = useQueryCache()
  const mutations = useMutationCache()
  const scope = useSessionScope()
  const preferences = usePreferencesStore()

  function clearPrivateData() {
    for (const entry of cache.getEntries({ key: ['private'] })) {
      cache.cancel(entry)
      cache.remove(entry)
    }
    for (const entry of mutations.getEntries({ key: ['private'] })) {
      mutations.remove(entry)
    }
    scope.advance()
    preferences.reset()
  }

  async function expire() {
    cache.cancelQueries({ key: SESSION_KEY })
    cache.setQueryData(SESSION_KEY, null)
    clearPrivateData()
    await navigateTo('/login')
  }

  let pendingCredentials: LoginInput | undefined
  const loginMutation = useMutation({
    gcTime: 0,
    mutation: async () => {
      const input = pendingCredentials
      pendingCredentials = undefined
      if (!input) throw new Error('Enter your credentials before signing in')

      try {
        return sessionSchema.parse(
          await request('/auth/login', {
            method: 'POST',
            body: input,
          }),
        )
      } finally {
        input.password = ''
      }
    },
    onSuccess: async (user) => {
      cache.cancelQueries({ key: SESSION_KEY })
      clearPrivateData()
      cache.setQueryData(SESSION_KEY, user)
      await navigateTo('/')
    },
  })

  // Credentials are never mutation variables: Colada retains variables in its cache.
  async function signIn(input: LoginInput) {
    if (loginMutation.asyncStatus.value === 'loading') {
      throw new Error('A sign-in request is already in progress')
    }
    pendingCredentials = input
    try {
      return await loginMutation.mutateAsync()
    } finally {
      pendingCredentials = undefined
      input.password = ''
    }
  }

  const login = { ...loginMutation, mutateAsync: signIn }

  const logout = useMutation({
    mutation: () => request('/auth/logout', { method: 'POST', body: {} }),
    onSuccess: expire,
  })

  return { session, login, logout, expire }
}
