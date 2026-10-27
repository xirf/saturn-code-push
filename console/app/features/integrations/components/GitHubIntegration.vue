<script setup lang="ts">
import { useQuery, useQueryCache } from '@pinia/colada'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { request, errorMessage } from '~/lib/api/client'
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
const busy = ref(false)
const error = ref('')
const notice = ref('')
async function action(path: string, body: unknown) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  notice.value = ''
  const generation = scope.generation
  try {
    await request(path, { method: 'POST', body })
    if (generation !== scope.generation) return
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
  <Card v-if="identity.session.data.value?.role === 'admin'">
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
          <form
            class="grid gap-4 sm:grid-cols-2"
            @submit.prevent="
              action('/admin/integrations/github', {
                ...form,
                installation_id: Number(form.installation_id),
              })
            "
          >
            <label class="space-y-2 text-sm">
              Application
              <select
                v-model="form.app_id"
                aria-label="GitHub application"
                required
                class="block w-full rounded-md border bg-background p-2"
              >
                <option
                  value=""
                  disabled
                >
                  Select an application
                </option>
                <option
                  v-for="item in workspace.applications.value"
                  :key="item.id"
                  :value="item.id"
                >
                  {{ item.display_name }}
                </option>
              </select>
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
          <Button
            variant="outline"
            :disabled="busy"
            @click="action('/admin/integrations/github/sync', {})"
          >
            Refresh linked runs
          </Button>
        </template>
        <div
          v-for="connection in query.data.value.connections"
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
