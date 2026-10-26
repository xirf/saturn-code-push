import { z } from '~/lib/validation'

export const releaseSchema = z.object({
  id: z.number(),
  app_id: z.string(),
  version: z.string(),
  flutter_revision: z.string(),
  flutter_version: z.string().nullable().optional(),
  platforms: z.record(z.string(), z.object({ status: z.string() }).passthrough()),
  created_at: z.string(),
})
export type Release = z.infer<typeof releaseSchema>
