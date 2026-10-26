import { z } from '~/lib/validation'

export const eventSchema = z.object({
  received_at: z.string(),
  event: z
    .object({
      app_id: z.string(),
      type: z.string(),
      release_version: z.string().optional(),
      patch_number: z.number().optional(),
      client_id: z.string().optional(),
      platform: z.string().optional(),
      arch: z.string().optional(),
      message: z.string().optional(),
    })
    .passthrough(),
})
export type DeviceEvent = z.infer<typeof eventSchema>
