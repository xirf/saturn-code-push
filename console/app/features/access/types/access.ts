import { z } from '~/lib/validation'
export const roleSchema = z.enum(['admin', 'publisher', 'viewer'])
export const accountSchema = z.object({
  id: z.string(),
  username: z.string(),
  role: roleSchema,
  disabled: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type Account = z.infer<typeof accountSchema>
export const accountsSchema = z.object({ accounts: z.array(accountSchema) })
export const keysSchema = z.object({
  keys: z.array(
    z.object({
      id: z.string(),
      name: z.string(),
      prefix: z.string(),
      app_id: z.string(),
      permission: z.string(),
      creator: z.string(),
      created_at: z.string(),
      expires_at: z.number(),
      revoked_at: z.string().nullable(),
      last_used_at: z.string().nullable(),
    }),
  ),
})
export const auditSchema = z.object({
  events: z.array(
    z.object({
      id: z.number(),
      actor_id: z.string(),
      actor_name: z.string(),
      actor_kind: z.string(),
      action: z.string(),
      resource: z.string(),
      app_id: z.string().nullable(),
      occurred_at: z.string(),
    }),
  ),
})
export const newKeySchema = z.object({ id: z.string(), token: z.string() })
