import { z } from '~/lib/validation'

export const applicationSchema = z.object({
  id: z.string(),
  display_name: z.string(),
  created_at: z.string(),
})
export type Application = z.infer<typeof applicationSchema>

export const createApplicationSchema = z.object({
  display_name: z
    .string()
    .trim()
    .min(1, 'Enter an application name')
    .refine((value) => new TextEncoder().encode(value).length <= 256, 'Use no more than 256 bytes'),
})
