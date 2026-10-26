<script setup lang="ts">
import { File, FileImage, FileText, Search, PackageOpen } from '@lucide/vue'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import { errorMessage } from '~/lib/api/client'
import { formatBytes } from '~/lib/format'
import { useAssetInspection } from '../composables/useAssetInspection'
import type { AssetBundle, AssetFile } from '../types/asset-bundle'

const props = defineProps<{ bundle: AssetBundle }>()
const search = ref('')
const selected = ref<AssetFile>()
const inspection = useAssetInspection(toRef(props, 'bundle'), selected)
const files = computed(() =>
  (inspection.files.data.value?.files ?? []).filter((file) =>
    file.path.toLowerCase().includes(search.value.toLowerCase()),
  ),
)

function icon(file: AssetFile) {
  if (file.preview === 'image') return FileImage
  if (file.preview === 'text') return FileText
  return File
}
</script>

<template>
  <div
    v-if="!bundle.uploaded"
    class="grid min-h-64 place-items-center p-8 text-center"
  >
    <div class="space-y-3">
      <PackageOpen class="mx-auto size-8 text-muted-foreground" />
      <h3 class="font-medium">Awaiting upload</h3>
      <p class="max-w-sm text-sm text-muted-foreground">
        This bundle is registered. Its files will be available after the ZIP finishes uploading.
      </p>
    </div>
  </div>
  <div
    v-else-if="inspection.files.error.value"
    class="p-6"
  >
    <ErrorNotice :message="errorMessage(inspection.files.error.value)">
      <Button
        class="mt-3"
        size="sm"
        variant="outline"
        @click="inspection.files.refetch()"
      >
        Try again
      </Button>
    </ErrorNotice>
  </div>
  <div
    v-else-if="!inspection.files.data.value"
    class="space-y-4 p-6"
    role="status"
    aria-label="Loading asset files"
  >
    <Skeleton class="h-10 w-full" />
    <Skeleton class="h-48 w-full" />
  </div>
  <div
    v-else
    class="grid min-h-0 grid-rows-[minmax(10rem,1fr)_minmax(12rem,1fr)] md:grid-cols-[20rem_minmax(0,1fr)] md:grid-rows-1"
  >
    <section
      class="flex min-h-0 flex-col border-b md:border-r md:border-b-0"
      aria-label="Asset files"
    >
      <div class="space-y-3 border-b p-4">
        <div class="flex items-center justify-between text-xs text-muted-foreground">
          <span>{{ inspection.files.data.value.files.length }} files</span>
          <span>{{ formatBytes(inspection.files.data.value.expanded_size) }} expanded</span>
        </div>
        <div class="relative">
          <Search
            class="pointer-events-none absolute top-2.5 left-3 size-4 text-muted-foreground"
          />
          <Input
            v-model="search"
            class="pl-9"
            aria-label="Search asset files"
            placeholder="Search files…"
          />
        </div>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto p-2">
        <button
          v-for="file in files"
          :key="file.index"
          type="button"
          :aria-pressed="selected?.index === file.index"
          :title="file.path"
          :class="[
            'flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left text-xs outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring',
            selected?.index === file.index && 'bg-accent text-accent-foreground',
          ]"
          @click="selected = file"
        >
          <component
            :is="icon(file)"
            class="size-4 shrink-0 text-muted-foreground"
          />
          <span class="min-w-0 flex-1 truncate">{{ file.path }}</span>
          <span class="shrink-0 text-[11px] text-muted-foreground">
            {{ formatBytes(file.size) }}
          </span>
        </button>
        <p
          v-if="!files.length"
          class="px-3 py-6 text-sm text-muted-foreground"
        >
          {{ search ? 'No matching files.' : 'This ZIP contains no files.' }}
        </p>
      </div>
    </section>
    <section
      class="flex min-h-0 min-w-0 flex-col"
      aria-label="Asset preview"
    >
      <template v-if="selected">
        <div class="space-y-1 border-b px-5 py-4">
          <h3 class="break-all text-sm font-medium">{{ selected.path }}</h3>
          <p class="text-xs text-muted-foreground">
            {{ selected.size.toLocaleString() }} bytes ·
            {{ formatBytes(selected.compressed_size) }} compressed
          </p>
        </div>
        <div class="min-h-0 flex-1 overflow-auto p-5">
          <p
            v-if="selected.preview === 'none'"
            class="text-sm leading-relaxed text-muted-foreground"
          >
            Preview is unavailable for this file type or size. Download the bundle to inspect the
            original file. Text previews support files up to 256 KiB; images support files up to 4
            MiB and 2048 × 2048 pixels.
          </p>
          <ErrorNotice
            v-else-if="inspection.preview.error.value"
            :message="errorMessage(inspection.preview.error.value)"
          />
          <div
            v-else-if="!inspection.preview.data.value"
            role="status"
            aria-label="Loading asset preview"
          >
            <Skeleton class="h-40 w-full" />
          </div>
          <img
            v-else-if="inspection.preview.data.value.kind === 'image'"
            :src="inspection.preview.data.value.content"
            :alt="`Preview of ${selected.path}`"
            class="mx-auto max-h-full max-w-full rounded-md border bg-background object-contain"
          />
          <pre
            v-else
            class="whitespace-pre-wrap break-all font-mono text-xs leading-relaxed"
            >{{ inspection.preview.data.value.content }}</pre>
        </div>
      </template>
      <div
        v-else
        class="grid flex-1 place-items-center p-8 text-center"
      >
        <div class="space-y-3">
          <FileImage class="mx-auto size-8 text-muted-foreground" />
          <h3 class="text-sm font-medium">Select a file to inspect</h3>
          <p class="max-w-xs text-xs leading-relaxed text-muted-foreground">
            Preview images and UTF-8 text. Fonts and compiled manifests are listed with their sizes.
          </p>
        </div>
      </div>
    </section>
  </div>
</template>
