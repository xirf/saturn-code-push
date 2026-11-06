<script setup lang="ts">
import { Search } from '@lucide/vue'
import { InputGroup, InputGroupAddon, InputGroupInput } from '~/components/ui/input-group'
import PageHeader from '~/components/shared/PageHeader.vue'
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
  <div class="space-y-6">
    <PageHeader
      title="Patches"
      description="Promote, roll back, or restore patches for this application."
    />
    <Card>
      <CardHeader>
        <div class="flex flex-wrap items-center justify-between gap-4">
          <CardTitle>
            All patches
            <span class="ml-2 text-muted-foreground">{{ workspace.patches.value.length }}</span>
          </CardTitle>
          <InputGroup class="w-full max-w-sm">
            <InputGroupInput
              v-model="search"
              class="pl-9"
              aria-label="Search patches"
              placeholder="Search patches…"
            />
            <InputGroupAddon><Search aria-hidden="true" /></InputGroupAddon>
          </InputGroup>
        </div>
      </CardHeader>
      <CardContent class="px-0"><PatchTable :patches="patches" /></CardContent>
      <CardFooter class="text-xs leading-relaxed text-muted-foreground">
        Promotion controls delivery. Rollbacks take effect after a device checks and restarts.
      </CardFooter>
    </Card>
  </div>
</template>
