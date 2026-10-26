import { z } from '~/lib/validation'

export const artifactSchema = z.object({
  id: z.number(),
  app_id: z.string(),
  release_id: z.number(),
  patch_id: z.number().nullable(),
  arch: z.string(),
  platform: z.string(),
  uploaded: z.boolean(),
  size: z.number().nonnegative(),
})
export type Artifact = z.infer<typeof artifactSchema>
