<script setup lang="ts">
import { useIdentity } from '~/features/identity/composables/useIdentity'
import type { Release } from '~/features/distribution/types/release'
import type { Patch } from '~/features/distribution/types/patch'
import { ApiError, request, errorMessage } from '~/lib/api/client'
import { comparisonUrl, githubUrl, previousPublished, type Deployment } from '../lib/links'

const props = defineProps<{ release: Release; patches?: Patch[] }>()
const identity = useIdentity()
const canPublish = computed(() =>
  ['admin', 'publisher'].includes(identity.session.data.value?.role ?? ''),
)
const items = ref<Deployment[]>([])
const error = ref('')
const busy = ref(false)
const baseline = ref('')
const form = reactive({
  patch_id: '',
  key: '',
  repository: '',
  commit: '',
  source_kind: 'commit',
  source_ref: '',
  destination: 'app-store/production',
  status: 'uploaded',
  build_number: '',
  workflow_run: '',
  workflow_attempt: '',
})
const statuses = ['pending', 'running', 'failed', 'uploaded', 'in_review', 'published']
const current = computed(() => items.value.filter((item) => item.release_id === props.release.id))
const baselineSelection = computed({
  get: () => baseline.value || 'previous',
  set: (value: string) => (baseline.value = value === 'previous' ? '' : value),
})
const patchSelection = computed({
  get: () => form.patch_id || 'release-baseline',
  set: (value: string) => (form.patch_id = value === 'release-baseline' ? '' : value),
})
const endpoint = computed(
  () => `/api/v1/apps/${encodeURIComponent(props.release.app_id)}/deployments`,
)
let generation = 0
let reading = 0
async function load() {
  if (busy.value) return
  const token = generation
  const read = ++reading
  try {
    const result = (await request(endpoint.value)) as Deployment[]
    if (token === generation && read === reading) {
      items.value = result
      error.value = ''
    }
  } catch (cause) {
    if (token === generation && read === reading) error.value = errorMessage(cause)
    if (cause instanceof ApiError && cause.status === 401) await identity.expire()
  }
}
watch(
  endpoint,
  () => {
    generation++
    items.value = []
    void load()
  },
  { immediate: true },
)
let timer: ReturnType<typeof setInterval> | undefined
onMounted(() => {
  timer = setInterval(() => {
    if (!document.hidden) void load()
  }, 15000)
})
onBeforeUnmount(() => {
  generation++
  reading++
  clearInterval(timer)
})
async function save() {
  busy.value = true
  reading++
  error.value = ''
  const token = generation
  try {
    const result = (await request(endpoint.value, {
      method: 'POST',
      body: {
        ...form,
        release_id: props.release.id,
        patch_id: form.patch_id ? Number(form.patch_id) : null,
        key: form.key || crypto.randomUUID(),
        workflow_run: form.workflow_run ? Number(form.workflow_run) : null,
        workflow_attempt: form.workflow_attempt ? Number(form.workflow_attempt) : null,
      },
    })) as Deployment[]
    if (token === generation) items.value = result
  } catch (cause) {
    if (token === generation) error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}
async function update(item: Deployment, status: string) {
  busy.value = true
  reading++
  error.value = ''
  const token = generation
  try {
    const result = (await request(`${endpoint.value}/${item.id}`, {
      method: 'PATCH',
      body: { status },
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
        <Select v-model="baselineSelection">
          <SelectTrigger
            aria-label="Compare against"
            class="w-full"
          >
            <SelectValue placeholder="Previous published release or patch for this target" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="previous">
              Previous published release or patch for this target
            </SelectItem>
            <SelectItem
              v-for="item in items"
              :key="item.id"
              :value="String(item.id)"
            >
              {{ item.patch_id == null ? 'Release' : `Patch ${item.patch_id}` }} ·
              {{ item.destination }} · {{ item.source_ref }} · {{ item.commit.slice(0, 7) }} ·
              {{ item.status }}
            </SelectItem>
          </SelectContent>
        </Select>
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
          {{ release.version }} ·
          {{
            item.patch_id == null
              ? 'Release baseline'
              : `Patch #${props.patches?.find((p) => p.id === item.patch_id)?.number ?? item.patch_id}`
          }}
          <span v-if="item.build_number">· Build {{ item.build_number }}</span>
          · {{ item.destination }}
        </p>
        <p class="text-sm">
          CI:
          {{
            item.workflow?.status?.replaceAll('_', ' ') ??
            (item.workflow_run ? 'Awaiting GitHub sync' : 'No workflow linked')
          }}
          <span v-if="item.workflow?.conclusion">
            · {{ item.workflow.conclusion.replaceAll('_', ' ') }}
          </span>
          <span v-if="item.workflow?.run_attempt">· Attempt {{ item.workflow.run_attempt }}</span>
        </p>
        <p
          v-if="item.workflow?.sync_error"
          role="status"
          class="text-xs text-destructive"
        >
          {{ item.workflow.sync_error }}. Last known CI status may be stale.
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
          <Select
            :model-value="item.status"
            :disabled="busy || !canPublish"
            @update:model-value="(value) => update(item, String(value ?? item.status))"
          >
            <SelectTrigger
              aria-label="Reported deployment status"
              class="w-auto min-w-36"
            >
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="status in statuses"
                :key="status"
                :value="status"
              >
                {{ status.replaceAll('_', ' ') }}
              </SelectItem>
            </SelectContent>
          </Select>
        </label>
        <p class="text-xs text-muted-foreground">
          Reported by {{ item.reported_by ?? 'Unknown (imported)' }} · Last updated by
          {{ item.updated_by ?? 'Unknown (imported)' }}
        </p>
        <div class="flex gap-4 text-sm">
          <a
            v-if="item.workflow_run"
            :href="`${githubUrl(item)}/actions/runs/${item.workflow_run}${item.workflow_attempt ? `/attempts/${item.workflow_attempt}` : ''}`"
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
          <label class="space-y-1 text-sm">
            Deployment target
            <Select v-model="patchSelection">
              <SelectTrigger
                aria-label="Deployment target"
                class="w-full"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="release-baseline">Release baseline</SelectItem>
                <SelectItem
                  v-for="patch in props.patches ?? []"
                  :key="patch.id"
                  :value="String(patch.id)"
                >
                  Patch #{{ patch.number }}
                </SelectItem>
              </SelectContent>
            </Select>
          </label>
          <label
            v-for="field in [
              'repository',
              'commit',
              'source_ref',
              'destination',
              'build_number',
              'workflow_run',
              'workflow_attempt',
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
                  workflow_attempt: 'Actions attempt (optional; blank tracks latest)',
                  key: 'Deployment key (optional; reuse to update)',
                }[field]
              }}
            </span>
            <Input
              v-model="form[field]"
              :list="field === 'destination' ? 'deployment-destinations' : undefined"
              :required="['repository', 'commit', 'source_ref', 'destination'].includes(field)"
              :maxlength="field === 'commit' ? 64 : 256"
              :type="field.startsWith('workflow_') ? 'number' : 'text'"
              :min="field.startsWith('workflow_') ? 1 : undefined"
            />
          </label>
          <p class="text-xs text-muted-foreground sm:col-span-2">
            Destinations can include app-store/testflight, google-play/internal,
            microsoft-store/production, or a custom track.
          </p>
          <label class="space-y-1 text-sm">
            GitHub source type
            <Select v-model="form.source_kind">
              <SelectTrigger
                aria-label="GitHub source type"
                class="w-full"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="kind in ['commit', 'tag', 'release']"
                  :key="kind"
                  :value="kind"
                >
                  {{ kind }}
                </SelectItem>
              </SelectContent>
            </Select>
          </label>
          <label class="space-y-1 text-sm">
            Reported status
            <Select v-model="form.status">
              <SelectTrigger
                aria-label="Reported status"
                class="w-full"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="status in statuses"
                  :key="status"
                  :value="status"
                >
                  {{ status.replaceAll('_', ' ') }}
                </SelectItem>
              </SelectContent>
            </Select>
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
