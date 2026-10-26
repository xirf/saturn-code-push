import { useQuery } from '@pinia/colada'
import type { Ref } from 'vue'
import { ApiError, request } from '~/lib/api/client'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import {
  assetInspectionSchema,
  assetPreviewSchema,
  type AssetBundle,
  type AssetFile,
} from '../types/asset-bundle'

export function assetBundlePath(bundle: AssetBundle) {
  return `/api/v1/apps/${encodeURIComponent(bundle.app_id)}/patches/${bundle.patch_id}/asset-bundles/${bundle.id}`
}

export function useAssetInspection(bundle: Ref<AssetBundle>, selected: Ref<AssetFile | undefined>) {
  const identity = useIdentity()
  const scope = useSessionScope()
  const key = computed(() => ['private', scope.generation, 'asset-bundles', bundle.value.id])
  const files = useQuery({
    key: () => [...key.value, 'files'],
    enabled: () => Boolean(identity.session.data.value) && bundle.value.uploaded,
    staleTime: Infinity,
    gcTime: 0,
    query: async ({ signal }) =>
      assetInspectionSchema.parse(
        await request(`${assetBundlePath(bundle.value)}/files`, { signal }),
      ),
  })
  const preview = useQuery({
    key: () => [...key.value, 'preview', selected.value?.index ?? -1],
    enabled: () =>
      Boolean(identity.session.data.value) &&
      Boolean(selected.value && selected.value.preview !== 'none'),
    staleTime: Infinity,
    gcTime: 0,
    query: async ({ signal }) => {
      const file = selected.value
      if (!file) throw new Error('Select a file to preview')
      return assetPreviewSchema.parse(
        await request(`${assetBundlePath(bundle.value)}/files/${file.index}/preview`, { signal }),
      )
    },
  })

  watch([files.error, preview.error], (errors) => {
    if (errors.some((error) => error instanceof ApiError && error.status === 401)) {
      void identity.expire()
    }
  })

  return { files, preview }
}
