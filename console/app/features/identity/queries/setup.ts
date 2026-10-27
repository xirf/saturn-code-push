import { defineQuery, useQuery } from '@pinia/colada'
import { request } from '~/lib/api/client'
import { z } from '~/lib/validation'
export const SETUP_KEY = ['setup'] as const
export const setupStatusSchema = z.object({ required: z.boolean(), enabled: z.boolean() })
export const useSetupQuery = defineQuery(() =>
  useQuery({
    key: SETUP_KEY,
    staleTime: 0,
    query: async ({ signal }) => setupStatusSchema.parse(await request('/auth/setup', { signal })),
  }),
)
