import { z } from '~/lib/validation'

export const assetBundleSchema = z.object({
  id: z.number(),
  app_id: z.string(),
  patch_id: z.number(),
  platform: z.string(),
  uploaded: z.boolean(),
  size: z.number().nonnegative(),
  sha256: z.string(),
})
export type AssetBundle = z.infer<typeof assetBundleSchema>

export const assetFileSchema = z.object({
  index: z.number().int().nonnegative(),
  path: z.string(),
  size: z.number().nonnegative(),
  compressed_size: z.number().nonnegative(),
  preview: z.enum(['image', 'text', 'none']),
})

export const assetInspectionSchema = z.object({
  files: z.array(assetFileSchema),
  expanded_size: z.number().nonnegative(),
})

export const assetPreviewSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('text'), content: z.string().max(256 * 1024) }),
  z.object({
    kind: z.literal('image'),
    content: z
      .string()
      .max(4 * 1024 * 1024)
      .startsWith('data:image/png;base64,'),
  }),
])

export type AssetFile = z.infer<typeof assetFileSchema>
