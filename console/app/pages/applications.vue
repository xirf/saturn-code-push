<script setup lang="ts">
import { ArrowUpRight, Copy, Search } from '@lucide/vue'
import { toast } from 'vue-sonner'
import EmptyState from '~/components/shared/EmptyState.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { usePreferencesStore } from '~/stores/preferences'
import { formatDate } from '~/lib/format'

definePageMeta({
  middleware: 'auth',
  title: 'Applications',
  description: 'Manage applications and their release baselines.',
})
const workspace = useWorkspace()
const preferences = usePreferencesStore()
const search = ref('')
const applications = computed(() =>
  workspace.applications.value.filter((app) =>
    app.display_name.toLowerCase().includes(search.value.toLowerCase()),
  ),
)

async function openApplication(id: string) {
  preferences.selectedAppId = id
  await navigateTo('/releases')
}

async function copy(id: string) {
  try {
    await navigator.clipboard.writeText(id)
    toast.success('Application ID copied')
  } catch {
    toast.error('Clipboard access is unavailable. Select and copy the ID manually.')
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="relative max-w-sm">
      <Search class="absolute top-2.5 left-3 size-4 text-muted-foreground" />
      <Input
        v-model="search"
        aria-label="Search applications"
        placeholder="Search applications…"
        class="pl-9"
      />
    </div>
    <Card
      v-if="applications.length"
      class="overflow-hidden py-0"
    >
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="pl-5">Application</TableHead>
            <TableHead>Application ID</TableHead>
            <TableHead>Releases</TableHead>
            <TableHead>Created</TableHead>
            <TableHead><span class="sr-only">Actions</span></TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="app in applications"
            :key="app.id"
          >
            <TableCell class="py-4 pl-5 font-medium">
              <button
                class="hover:underline underline-offset-4"
                @click="openApplication(app.id)"
              >
                {{ app.display_name }}
              </button>
            </TableCell>
            <TableCell>
              <div class="flex items-center gap-2">
                <code class="text-xs text-muted-foreground">{{ app.id }}</code>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  :aria-label="`Copy ID for ${app.display_name}`"
                  @click="copy(app.id)"
                >
                  <Copy class="size-3.5" />
                </Button>
              </div>
            </TableCell>
            <TableCell class="tabular-nums">
              {{
                workspace.query.data.value?.releases.filter((release) => release.app_id === app.id)
                  .length ?? 0
              }}
            </TableCell>
            <TableCell class="text-muted-foreground">{{ formatDate(app.created_at) }}</TableCell>
            <TableCell class="pr-5 text-right">
              <Button
                variant="ghost"
                size="icon-sm"
                :aria-label="`View releases for ${app.display_name}`"
                @click="openApplication(app.id)"
              >
                <ArrowUpRight class="size-4" />
              </Button>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </Card>
    <Card v-else>
      <EmptyState
        :title="search ? 'No matching applications' : 'No applications yet'"
        description="Create an application to start a dedicated release pipeline."
      />
    </Card>
  </div>
</template>
