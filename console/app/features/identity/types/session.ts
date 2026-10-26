import { z } from '~/lib/validation'

export const sessionSchema = z.object({ username: z.string() })
export type Session = z.infer<typeof sessionSchema>

export const loginSchema = z.object({
  username: z.string().min(1, 'Enter your username').max(128),
  password: z.string().min(1, 'Enter your password').max(1024),
})
export type LoginInput = z.infer<typeof loginSchema>
