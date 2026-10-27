import { z } from '~/lib/validation'

export const patchSchema = z.object({
  id: z.number(),
  release_id: z.number(),
  number: z.number(),
  channel_ids: z.array(z.number()),
  rollout_percentages: z.record(z.string(), z.number()).default({}),
  is_rolled_back: z.boolean(),
  created_at: z.string(),
})
export type Patch = z.infer<typeof patchSchema>
export type PatchAction = 'promote' | 'rollback' | 'restore'
