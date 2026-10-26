<script setup lang="ts">
import { BookOpen, Copy, Download, FolderOpen } from '@lucide/vue'
import { toast } from 'vue-sonner'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { formatBytes } from '~/lib/format'
import { assetBundlePath } from '../composables/useAssetInspection'
import AssetBundleBrowser from './AssetBundleBrowser.vue'

const props = defineProps<{ patchId: number; patchNumber: number }>()
const workspace = useWorkspace()
const open = ref(false)
const bundleId = ref<string>()
const bundles = computed(() =>
  workspace.bundles.value.filter((bundle) => bundle.patch_id === props.patchId),
)
const selected = computed(
  () => bundles.value.find((bundle) => String(bundle.id) === bundleId.value) ?? bundles.value[0],
)

watch(open, (value) => {
  if (!value) bundleId.value = undefined
})

async function copyHash() {
  if (!selected.value) return
  try {
    await navigator.clipboard.writeText(selected.value.sha256)
    toast.success('Bundle SHA-256 copied')
  } catch {
    toast.error('Clipboard access is unavailable. Select and copy the hash manually.')
  }
}
</script>

<template>
  <span
    v-if="!bundles.length"
    class="text-xs text-muted-foreground"
  >
    No asset bundle
  </span>
  <Dialog
    v-else
    v-model:open="open"
  >
    <DialogTrigger as-child>
      <Button
        variant="ghost"
        size="sm"
        class="h-auto justify-start px-2 py-1.5 text-xs"
        :aria-label="`Inspect assets for patch ${patchNumber}`"
      >
        <FolderOpen class="size-3.5" />
        {{ bundles.length === 1 ? bundles[0]?.platform : `${bundles.length} platforms` }}
        <span class="text-muted-foreground">
          {{ bundles.some((bundle) => !bundle.uploaded) ? '· pending' : '· inspect' }}
        </span>
      </Button>
    </DialogTrigger>
    <DialogContent
      class="grid h-[min(52rem,90svh)] grid-rows-[auto_minmax(0,1fr)] gap-0 overflow-hidden p-0 sm:max-w-6xl"
    >
      <div class="space-y-4 border-b px-5 py-5 sm:px-6">
        <DialogHeader class="pr-8 text-left">
          <DialogTitle>Assets for patch #{{ patchNumber }}</DialogTitle>
          <DialogDescription>
            Files included in this patch’s overlay. Unchanged assets remain in the installed
            application.
          </DialogDescription>
        </DialogHeader>
        <Button
          as-child
          size="sm"
          variant="link"
          class="h-auto justify-start p-0 text-xs"
        >
          <NuxtLink to="/docs/asset-updates">
            <BookOpen class="size-3.5" />
            Asset update guide
          </NuxtLink>
        </Button>
        <template v-if="selected">
          <div class="flex flex-wrap items-center gap-3">
            <Select
              :model-value="String(selected.id)"
              @update:model-value="(value) => (bundleId = String(value))"
            >
              <SelectTrigger
                aria-label="Asset bundle platform"
                class="w-40"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="bundle in bundles"
                  :key="bundle.id"
                  :value="String(bundle.id)"
                >
                  {{ bundle.platform }}
                </SelectItem>
              </SelectContent>
            </Select>
            <Badge variant="outline">
              {{ selected.uploaded ? 'Uploaded' : 'Awaiting upload' }}
            </Badge>
            <span class="text-xs text-muted-foreground">{{ formatBytes(selected.size) }} ZIP</span>
            <Button
              v-if="selected.uploaded"
              as-child
              variant="outline"
              size="sm"
              class="ml-auto"
            >
              <a
                :href="`${assetBundlePath(selected)}/download`"
                download
              >
                <Download class="size-3.5" />
                Download ZIP
              </a>
            </Button>
          </div>
          <div class="flex items-start gap-2 text-[11px] text-muted-foreground">
            <span class="shrink-0 pt-1.5">SHA-256</span>
            <code class="min-w-0 break-all pt-1.5">{{ selected.sha256 }}</code>
            <Button
              variant="ghost"
              size="icon-sm"
              class="shrink-0"
              aria-label="Copy bundle SHA-256"
              @click="copyHash"
            >
              <Copy class="size-3" />
            </Button>
          </div>
        </template>
      </div>
      <AssetBundleBrowser
        v-if="open && selected"
        :key="selected.id"
        :bundle="selected"
        class="min-h-0"
      />
    </DialogContent>
  </Dialog>
</template>
