import { z } from '~/lib/validation'

export const channelSchema = z.object({ id: z.number(), app_id: z.string(), name: z.string() })
export type Channel = z.infer<typeof channelSchema>
