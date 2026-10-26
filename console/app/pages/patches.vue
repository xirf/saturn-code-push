<script setup lang="ts">
import { Search } from '@lucide/vue'
import PatchTable from '~/features/distribution/components/PatchTable.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'

definePageMeta({
  middleware: 'auth',
  title: 'Patches',
  description: 'Published patches, delivery channels, and rollback state.',
})
const workspace = useWorkspace()
const search = ref('')
const patches = computed(() =>
  workspace.patches.value.filter((patch) => {
    const version =
      workspace.releases.value.find((release) => release.id === patch.release_id)?.version ?? ''
    return `${patch.number} ${version} ${patch.is_rolled_back ? 'rolled back' : 'available'}`
      .toLowerCase()
      .includes(search.value.toLowerCase())
  }),
)
</script>

<template>
  <Card>
    <CardHeader>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <CardTitle>
            All patches
            <span class="ml-2 text-muted-foreground">{{ workspace.patches.value.length }}</span>
          </CardTitle>
          <CardDescription class="mt-1">Promote, roll back, or restore a patch.</CardDescription>
        </div>
        <div class="relative">
          <Search class="absolute top-2.5 left-3 size-4 text-muted-foreground" />
          <Input
            v-model="search"
            aria-label="Search patches"
            placeholder="Search patches…"
            class="pl-9"
          />
        </div>
      </div>
    </CardHeader>
    <CardContent class="px-0"><PatchTable :patches="patches" /></CardContent>
    <CardFooter class="text-xs leading-relaxed text-muted-foreground">
      Promotion controls delivery. Rollbacks take effect after a device checks and restarts.
    </CardFooter>
  </Card>
</template>
