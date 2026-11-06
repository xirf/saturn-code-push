<script setup lang="ts">
import { useQuery, useQueryCache } from '@pinia/colada'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { request, errorMessage } from '~/lib/api/client'
import { ScrollArea } from '~/components/ui/scroll-area'
import { z } from '~/lib/validation'
const schema = z.object({
  configured: z.boolean(),
  installation_url: z.string().nullable(),
  webhook_url: z.string(),
  connections: z.array(
    z.object({
      app_id: z.string(),
      repository: z.string(),
      installation_id: z.number(),
      connected_at: z.string(),
    }),
  ),
})
const scope = useSessionScope()
const identity = useIdentity()
const workspace = useWorkspace()
const cache = useQueryCache()
const query = useQuery({
  key: () => ['private', scope.generation, 'github-integration'],
  enabled: () => identity.session.data.value?.role === 'admin',
  query: async ({ signal }) =>
    schema.parse(await request('/admin/integrations/github', { signal })),
})
const form = reactive({ app_id: '', repository: '', installation_id: '' })
// Inside /apps/:appId the connection is for that application only.
const scoped = computed(() => Boolean(workspace.appId.value))
watch(
  () => workspace.appId.value,
  (id) => {
    if (id) form.app_id = id
  },
  { immediate: true },
)
const connections = computed(() =>
  (query.data.value?.connections ?? []).filter(
    (item) => !scoped.value || item.app_id === workspace.appId.value,
  ),
)
const busy = ref(false)
const error = ref('')
const notice = ref('')
const connectionOpen = ref(false)
async function action(path: string, body: unknown) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  notice.value = ''
  const generation = scope.generation
  try {
    await request(path, { method: 'POST', body })
    if (generation !== scope.generation) return
    if (path === '/admin/integrations/github') connectionOpen.value = false
    await cache.invalidateQueries({ key: ['private', generation] })
    notice.value = path.endsWith('/sync')
      ? 'Refresh queued. Linked runs will update shortly.'
      : 'GitHub connection updated.'
  } catch (cause) {
    if (generation === scope.generation) error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}
</script>
<template>
  <Card
    v-if="identity.session.data.value?.role === 'admin'"
    id="github"
    class="scroll-mt-6"
  >
    <CardHeader>
      <CardTitle>GitHub integration</CardTitle>
      <CardDescription>
        Connect a GitHub App to sync Actions runs. CI success is separate from publishing a release
        or patch.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-5">
      <p
        v-if="error || query.error.value"
        role="alert"
        class="text-sm text-destructive"
      >
        {{ error || errorMessage(query.error.value) }}
      </p>
      <p
        v-if="notice"
        role="status"
        class="text-sm"
      >
        {{ notice }}
      </p>
      <template v-if="query.data.value">
        <p
          v-if="!query.data.value.configured"
          class="text-sm text-muted-foreground"
        >
          Configure the GitHub App ID, slug, private key file, and webhook secret in deployment
          settings to enable syncing.
        </p>
        <template v-else>
          <a
            :href="query.data.value.installation_url ?? undefined"
            target="_blank"
            rel="noopener noreferrer"
            class="text-sm underline"
          >
            Install GitHub App on selected repositories
          </a>
          <p class="text-sm text-muted-foreground">
            After installation, copy the installation ID from its GitHub settings URL. The server
            verifies that the installation can access the repository you select.
          </p>
          <p class="break-all text-xs text-muted-foreground">
            Webhook URL: {{ query.data.value.webhook_url }}
          </p>
          <Button @click="connectionOpen = true">Connect a repository</Button>
          <Dialog v-model:open="connectionOpen">
            <DialogContent class="max-h-[90svh] overflow-hidden">
              <ScrollArea class="max-h-[80svh] w-full pr-3">
                <DialogHeader>
                  <DialogTitle>Connect a repository</DialogTitle>
                  <DialogDescription>
                    Link an application to a repository the GitHub App can access.
                  </DialogDescription>
                </DialogHeader>
                <form
                  class="grid gap-4"
                  @submit.prevent="
                    action('/admin/integrations/github', {
                      ...form,
                      installation_id: Number(form.installation_id),
                    })
                  "
                >
                  <label
                    v-if="!scoped"
                    class="space-y-2 text-sm"
                  >
                    Application
                    <Select
                      :model-value="form.app_id"
                      @update:model-value="(value) => (form.app_id = String(value ?? ''))"
                    >
                      <SelectTrigger
                        aria-label="GitHub application"
                        class="w-full"
                      >
                        <SelectValue placeholder="Select an application" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem
                          v-for="item in workspace.applications.value"
                          :key="item.id"
                          :value="item.id"
                        >
                          {{ item.display_name }}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </label>
                  <label class="space-y-2 text-sm">
                    GitHub repository
                    <Input
                      v-model="form.repository"
                      placeholder="owner/repository"
                      required
                      maxlength="256"
                    />
                  </label>
                  <label class="space-y-2 text-sm">
                    Installation ID
                    <Input
                      v-model="form.installation_id"
                      type="number"
                      min="1"
                      required
                    />
                  </label>
                  <Button
                    type="submit"
                    :disabled="busy || !form.app_id"
                  >
                    Connect repository
                  </Button>
                </form>
                <p
                  v-if="error"
                  role="alert"
                  class="text-sm text-destructive"
                >
                  {{ error }}
                </p>
              </ScrollArea>
            </DialogContent>
          </Dialog>
          <Button
            variant="outline"
            :disabled="busy"
            @click="action('/admin/integrations/github/sync', {})"
          >
            Refresh linked runs
          </Button>
        </template>
        <p
          v-if="scoped && !connections.length"
          class="rounded-lg border border-dashed px-4 py-6 text-center text-sm text-muted-foreground"
        >
          This application is not linked to a repository yet.
        </p>
        <div
          v-for="connection in connections"
          :key="connection.app_id"
          class="flex flex-wrap items-center justify-between gap-3 rounded-md border p-3 text-sm"
        >
          <div>
            {{
              workspace.applications.value.find((item) => item.id === connection.app_id)
                ?.display_name ?? connection.app_id
            }}
            · {{ connection.repository }}
            <p class="text-xs text-muted-foreground">
              Installation {{ connection.installation_id }}
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            :disabled="busy"
            :aria-label="`Disconnect ${connection.repository}`"
            @click="
              action(
                `/admin/integrations/github/${encodeURIComponent(connection.app_id)}/disconnect`,
                {},
              )
            "
          >
            Disconnect
          </Button>
        </div>
      </template>
      <NuxtLink
        to="/docs/github-deployments"
        class="block text-sm underline"
      >
        GitHub App setup and release/patch reporting guide
      </NuxtLink>
    </CardContent>
  </Card>
</template>
