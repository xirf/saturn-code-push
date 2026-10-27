<script setup lang="ts">
import { ArrowUpRight, Copy, Search } from '@lucide/vue'
import { toast } from 'vue-sonner'
import { InputGroup, InputGroupAddon, InputGroupInput } from '~/components/ui/input-group'
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
  <Card>
    <CardHeader>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <CardTitle>
            All applications
            <span class="ml-2 text-muted-foreground">
              {{ workspace.applications.value.length }}
            </span>
          </CardTitle>
          <CardDescription class="mt-1">
            Select an application to open its release history.
          </CardDescription>
        </div>
        <InputGroup class="w-full max-w-sm">
          <InputGroupInput
            v-model="search"
            class="pl-9"
            aria-label="Search applications"
            placeholder="Search applications…"
          />
          <InputGroupAddon><Search aria-hidden="true" /></InputGroupAddon>
        </InputGroup>
      </div>
    </CardHeader>
    <CardContent class="px-0">
      <Table v-if="applications.length">
        <TableHeader>
          <TableRow>
            <TableHead>Application</TableHead>
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
            <TableCell class="font-medium">
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
            <TableCell class="text-right">
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
      <EmptyState
        v-else
        class="px-5 py-8"
        :title="search ? 'No matching applications' : 'No applications yet'"
        description="Create an application to start a dedicated release pipeline."
      />
    </CardContent>
    <CardFooter class="text-xs text-muted-foreground">
      Application IDs are used by your CLI and deployment keys.
    </CardFooter>
  </Card>
</template>
