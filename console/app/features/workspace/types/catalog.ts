import { z } from '~/lib/validation'
import { applicationSchema } from '../../applications/types/application'
import { channelSchema } from '../../applications/types/channel'
import { releaseSchema } from '../../distribution/types/release'
import { patchSchema } from '../../distribution/types/patch'
import { artifactSchema } from '../../artifacts/types/artifact'
import { assetBundleSchema } from '../../artifacts/types/asset-bundle'
import { eventSchema } from '../../telemetry/types/event'

export const catalogSchema = z.object({
  apps: z.array(applicationSchema),
  channels: z.array(channelSchema),
  releases: z.array(releaseSchema),
  patches: z.array(patchSchema),
  artifacts: z.array(artifactSchema),
  asset_bundles: z.array(assetBundleSchema).default([]),
  events: z.array(eventSchema),
})
export type Catalog = z.infer<typeof catalogSchema>
