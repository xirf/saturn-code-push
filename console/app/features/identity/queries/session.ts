import { defineQuery, useQuery } from '@pinia/colada'
import { ApiError, request } from '~/lib/api/client'
import { sessionSchema } from '../types/session'

export const SESSION_KEY = ['session'] as const

export const useSessionQuery = defineQuery(() =>
  useQuery({
    key: SESSION_KEY,
    staleTime: 30_000,
    query: async ({ signal }) => {
      try {
        return sessionSchema.parse(await request('/auth/session', { signal }))
      } catch (error) {
        if (error instanceof ApiError && error.status === 401) return null
        throw error
      }
    },
  }),
)
