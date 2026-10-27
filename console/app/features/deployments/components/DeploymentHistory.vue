<script setup lang="ts">
import { useIdentity } from '~/features/identity/composables/useIdentity'
import type { Release } from '~/features/distribution/types/release'
import { request, errorMessage } from '~/lib/api/client'
import { comparisonUrl, githubUrl, previousPublished, type Deployment } from '../lib/links'

const props = defineProps<{ release: Release }>()
const identity = useIdentity()
const canPublish = computed(() =>
  ['admin', 'publisher'].includes(identity.session.data.value?.role ?? ''),
)
const items = ref<Deployment[]>([])
const error = ref('')
const busy = ref(false)
const baseline = ref('')
const form = reactive({
  key: '',
  repository: '',
  commit: '',
  source_kind: 'commit',
  source_ref: '',
  destination: 'app-store/production',
  status: 'uploaded',
  build_number: '',
  workflow_run: '',
})
const statuses = ['pending', 'running', 'failed', 'uploaded', 'in_review', 'published']
const current = computed(() => items.value.filter((item) => item.release_id === props.release.id))
const endpoint = computed(
  () => `/api/v1/apps/${encodeURIComponent(props.release.app_id)}/deployments`,
)
let generation = 0
watch(
  endpoint,
  async () => {
    const token = ++generation
    items.value = []
    error.value = ''
    try {
      const result = (await request(endpoint.value)) as Deployment[]
      if (token === generation) items.value = result
    } catch (cause) {
      if (token === generation) error.value = errorMessage(cause)
    }
  },
  { immediate: true },
)
onBeforeUnmount(() => {
  generation++
})
async function save() {
  busy.value = true
  error.value = ''
  const token = generation
  try {
    const result = (await request(endpoint.value, {
      method: 'POST',
      body: {
        ...form,
        release_id: props.release.id,
        key: form.key || crypto.randomUUID(),
        workflow_run: form.workflow_run ? Number(form.workflow_run) : null,
      },
    })) as Deployment[]
    if (token === generation) items.value = result
  } catch (cause) {
    if (token === generation) error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}
async function update(item: Deployment, event: Event) {
  busy.value = true
  error.value = ''
  const token = generation
  try {
    const result = (await request(`${endpoint.value}/${item.id}`, {
      method: 'PATCH',
      body: { status: (event.target as HTMLSelectElement).value },
    })) as Deployment[]
    if (token === generation) items.value = result
  } catch (cause) {
    if (token === generation) error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}
function compare(item: Deployment) {
  return comparisonUrl(
    baseline.value
      ? items.value.find((entry) => String(entry.id) === baseline.value)
      : previousPublished(items.value, item),
    item,
  )
}
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Deployments &amp; CI/CD</CardTitle>
      <CardDescription>
        Manually recorded or reported by CI. Store status is reported, not synced from store APIs.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-5">
      <p
        v-if="error"
        role="alert"
        class="text-destructive"
      >
        {{ error }}
      </p>
      <label class="block space-y-2">
        <span>Compare against</span>
        <select
          v-model="baseline"
          class="w-full rounded-md border bg-background p-2"
        >
          <option value="">Previous published deployment to the same destination</option>
          <option
            v-for="item in items"
            :key="item.id"
            :value="String(item.id)"
          >
            {{ item.destination }} · {{ item.source_ref }} · {{ item.commit.slice(0, 7) }} ·
            {{ item.status }}
          </option>
        </select>
      </label>
      <p
        v-if="!current.length"
        class="text-sm text-muted-foreground"
      >
        No deployments linked to this release.
      </p>
      <div
        v-for="item in current"
        :key="item.id"
        class="space-y-2 rounded-md border p-4"
      >
        <p class="font-medium">
          {{ release.version }}
          <span v-if="item.build_number">· Build {{ item.build_number }}</span>
          · {{ item.destination }}
        </p>
        <p class="text-sm">
          From {{ item.source_kind }} {{ item.source_ref }} ·
          <a
            :href="`${githubUrl(item)}/commit/${item.commit}`"
            target="_blank"
            rel="noopener noreferrer"
            class="underline"
          >
            {{ item.commit.slice(0, 7) }}
          </a>
        </p>
        <label class="flex items-center gap-2 text-sm">
          Reported status
          <select
            :value="item.status"
            :disabled="busy || !canPublish"
            class="rounded-md border bg-background p-2"
            @change="update(item, $event)"
          >
            <option
              v-for="status in statuses"
              :key="status"
              :value="status"
            >
              {{ status.replaceAll('_', ' ') }}
            </option>
          </select>
        </label>
        <p class="text-xs text-muted-foreground">
          Reported by {{ item.reported_by ?? 'Unknown (imported)' }} · Last updated by
          {{ item.updated_by ?? 'Unknown (imported)' }}
        </p>
        <div class="flex gap-4 text-sm">
          <a
            v-if="item.workflow_run"
            :href="`${githubUrl(item)}/actions/runs/${item.workflow_run}`"
            target="_blank"
            rel="noopener noreferrer"
            class="underline"
          >
            View workflow
          </a>
          <a
            v-if="item.source_kind === 'release'"
            :href="`${githubUrl(item)}/releases/tag/${encodeURIComponent(item.source_ref)}`"
            target="_blank"
            rel="noopener noreferrer"
            class="underline"
          >
            View GitHub release
          </a>
          <a
            v-if="compare(item)"
            :href="compare(item)"
            target="_blank"
            rel="noopener noreferrer"
            class="underline"
          >
            View changes
          </a>
          <span
            v-else
            class="text-muted-foreground"
          >
            Select a different deployment from the same repository to compare.
          </span>
        </div>
      </div>
      <NuxtLink
        to="/docs/github-deployments"
        class="block text-sm underline"
      >
        Set up GitHub Actions reporting
      </NuxtLink>
      <details v-if="canPublish">
        <summary class="cursor-pointer font-medium">Link a deployment manually</summary>
        <form
          class="mt-4 grid gap-3 sm:grid-cols-2"
          @submit.prevent="save"
        >
          <label
            v-for="field in [
              'repository',
              'commit',
              'source_ref',
              'destination',
              'build_number',
              'workflow_run',
              'key',
            ] as const"
            :key="field"
            class="space-y-1 text-sm"
          >
            <span>
              {{
                {
                  repository: 'GitHub repository (owner/repo)',
                  commit: 'Full commit SHA',
                  source_ref: 'Branch, tag or release tag',
                  destination: 'Destination',
                  build_number: 'Build number (optional)',
                  workflow_run: 'Actions run ID (optional)',
                  key: 'Deployment key (optional; reuse to update)',
                }[field]
              }}
            </span>
            <Input
              v-model="form[field]"
              :list="field === 'destination' ? 'deployment-destinations' : undefined"
              :required="['repository', 'commit', 'source_ref', 'destination'].includes(field)"
              :maxlength="field === 'commit' ? 64 : 256"
              :type="field === 'workflow_run' ? 'number' : 'text'"
              :min="field === 'workflow_run' ? 1 : undefined"
            />
          </label>
          <p class="text-xs text-muted-foreground sm:col-span-2">
            Destinations can include app-store/testflight, google-play/internal,
            microsoft-store/production, or a custom track.
          </p>
          <label class="space-y-1 text-sm">
            Source type
            <select
              v-model="form.source_kind"
              class="block w-full rounded-md border bg-background p-2"
            >
              <option
                v-for="kind in ['commit', 'tag', 'release']"
                :key="kind"
                :value="kind"
              >
                {{ kind }}
              </option>
            </select>
          </label>
          <label class="space-y-1 text-sm">
            Reported status
            <select
              v-model="form.status"
              class="block w-full rounded-md border bg-background p-2"
            >
              <option
                v-for="status in statuses"
                :key="status"
                :value="status"
              >
                {{ status.replaceAll('_', ' ') }}
              </option>
            </select>
          </label>
          <datalist id="deployment-destinations">
            <option value="app-store/testflight" />
            <option value="app-store/production" />
            <option value="google-play/internal" />
            <option value="google-play/production" />
            <option value="microsoft-store/production" />
          </datalist>
          <Button
            type="submit"
            :disabled="busy || !canPublish"
          >
            {{ busy ? 'Saving…' : 'Save deployment' }}
          </Button>
        </form>
      </details>
    </CardContent>
  </Card>
</template>
