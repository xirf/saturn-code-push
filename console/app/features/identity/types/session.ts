import { z } from '~/lib/validation'

export const sessionSchema = z.object({
  id: z.string(),
  username: z.string(),
  role: z.enum(['admin', 'publisher', 'viewer']),
})
export type Session = z.infer<typeof sessionSchema>

export const loginSchema = z.object({
  username: z.string().min(1, 'Enter your username').max(128),
  password: z.string().min(1, 'Enter your password').max(1024),
})
export type LoginInput = z.infer<typeof loginSchema>

export const setupSchema = z.object({
  setup_token: z.string().min(1, 'Enter the setup token from deployment settings').max(256),
  username: z.string().trim().min(1, 'Enter your username').max(128),
  password: z.string().min(12, 'Use at least 12 characters').max(1024),
})
export type SetupInput = z.infer<typeof setupSchema>
