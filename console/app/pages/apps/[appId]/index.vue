<script setup lang="ts">
import { ArrowUpRight, GitBranch, Layers } from '@lucide/vue'
import PageHeader from '~/components/shared/PageHeader.vue'
import EventStats from '~/features/telemetry/components/EventStats.vue'
import ActivityChart from '~/features/telemetry/components/ActivityChart.vue'
import ReleaseTable from '~/features/distribution/components/ReleaseTable.vue'
import PatchTable from '~/features/distribution/components/PatchTable.vue'
import GettingStarted from '~/features/workspace/components/GettingStarted.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { formatDate } from '~/lib/format'

definePageMeta({
  middleware: 'auth',
  title: 'Overview',
  description: 'Release health and delivery for this application.',
})
const workspace = useWorkspace()
const latestRelease = computed(() => workspace.releases.value[0])
const releasePatches = computed(() =>
  workspace.patches.value.filter((patch) => patch.release_id === latestRelease.value?.id),
)
// Prefer what production devices receive; fall back to anything promoted (e.g. staging only).
const stableId = computed(
  () => workspace.channels.value.find((channel) => channel.name === 'stable')?.id,
)
const promoted = computed(() =>
  releasePatches.value.filter((patch) => !patch.is_rolled_back && patch.channel_ids.length),
)
const livePatch = computed(
  () =>
    promoted.value.find((patch) => stableId.value && patch.channel_ids.includes(stableId.value)) ??
    promoted.value[0],
)
const liveLabel = computed(() =>
  livePatch.value && stableId.value && livePatch.value.channel_ids.includes(stableId.value)
    ? 'Live patch'
    : 'Latest promoted patch',
)
const liveChannels = computed(() =>
  (livePatch.value?.channel_ids ?? []).map((id) => {
    const name = workspace.channels.value.find((channel) => channel.id === id)?.name ?? 'Unknown'
    const percentage = livePatch.value?.rollout_percentages[String(id)] ?? 100
    return percentage === 100 ? name : `${name} · ${percentage === 0 ? 'Paused' : `${percentage}%`}`
  }),
)
const hasActivity = computed(() => workspace.releases.value.length > 0)
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      :title="workspace.application.value?.display_name ?? 'Overview'"
      description="Release health and delivery for this application."
    >
      <Button
        variant="outline"
        as-child
      >
        <NuxtLink :to="workspace.path('patches')">
          <GitBranch class="size-4" />
          Manage patches
        </NuxtLink>
      </Button>
    </PageHeader>

    <GettingStarted v-if="!hasActivity" />

    <template v-if="latestRelease">
      <Card class="gap-0 py-0">
        <header class="flex flex-wrap items-center justify-between gap-3 border-b px-6 py-4">
          <div>
            <h2 class="text-sm font-medium">Current release</h2>
            <p class="text-xs text-muted-foreground">
              The newest baseline published for this application.
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            as-child
          >
            <NuxtLink :to="workspace.path(`releases/${latestRelease.id}`)">
              View release
              <ArrowUpRight class="size-3.5" />
            </NuxtLink>
          </Button>
        </header>
        <div class="grid gap-8 px-6 py-6 md:grid-cols-[14rem_minmax(0,1fr)]">
          <div class="flex flex-col justify-between gap-4 rounded-lg border bg-muted/30 p-4">
            <Layers
              class="size-5 text-muted-foreground"
              aria-hidden="true"
            />
            <div>
              <p class="font-heading text-3xl font-semibold tracking-tight break-all">
                {{ latestRelease.version }}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                Published {{ formatDate(latestRelease.created_at) }}
              </p>
            </div>
          </div>
          <dl class="grid gap-x-8 gap-y-5 text-sm sm:grid-cols-2">
            <div class="space-y-1.5">
              <dt class="text-xs text-muted-foreground">Platforms</dt>
              <dd class="flex flex-wrap gap-2">
                <Badge
                  v-for="(platform, name) in latestRelease.platforms"
                  :key="name"
                  variant="outline"
                >
                  <span
                    :class="platform.status === 'active' ? 'bg-emerald-400' : 'bg-muted-foreground'"
                    class="size-1.5 rounded-full"
                  />
                  {{ name }} · {{ platform.status }}
                </Badge>
                <span
                  v-if="!Object.keys(latestRelease.platforms).length"
                  class="text-muted-foreground"
                >
                  Not activated
                </span>
              </dd>
            </div>
            <div class="space-y-1.5">
              <dt class="text-xs text-muted-foreground">{{ liveLabel }}</dt>
              <dd v-if="livePatch">
                <span class="font-medium">Patch #{{ livePatch.number }}</span>
                <span class="ml-2 inline-flex flex-wrap gap-1.5 align-middle">
                  <Badge
                    v-for="channel in liveChannels"
                    :key="channel"
                    variant="secondary"
                  >
                    {{ channel }}
                  </Badge>
                </span>
              </dd>
              <dd
                v-else
                class="text-muted-foreground"
              >
                None yet · devices run the baseline
              </dd>
            </div>
            <div class="space-y-1.5">
              <dt class="text-xs text-muted-foreground">Flutter revision</dt>
              <dd
                class="truncate font-mono text-xs"
                :title="latestRelease.flutter_revision"
              >
                {{ latestRelease.flutter_revision }}
              </dd>
            </div>
            <div class="space-y-1.5">
              <dt class="text-xs text-muted-foreground">Patches · channels</dt>
              <dd>{{ releasePatches.length }} · {{ workspace.channels.value.length }}</dd>
            </div>
          </dl>
        </div>
        <footer class="border-t px-6 py-3 text-xs text-muted-foreground">
          Promote a patch to a channel to deliver it. Rollbacks reach devices after their next check
          and restart.
        </footer>
      </Card>

      <EventStats />
      <ActivityChart />

      <div class="space-y-6">
        <Card>
          <CardHeader>
            <div class="flex items-center justify-between gap-3">
              <CardTitle>Recent patches</CardTitle>
              <Button
                variant="ghost"
                size="sm"
                as-child
              >
                <NuxtLink :to="workspace.path('patches')">
                  View all
                  <ArrowUpRight class="size-4" />
                </NuxtLink>
              </Button>
            </div>
          </CardHeader>
          <CardContent class="px-0">
            <PatchTable :patches="workspace.patches.value.slice(0, 3)" />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <div class="flex items-center justify-between gap-3">
              <CardTitle>Recent releases</CardTitle>
              <Button
                variant="ghost"
                size="sm"
                as-child
              >
                <NuxtLink :to="workspace.path('releases')">
                  View all
                  <ArrowUpRight class="size-4" />
                </NuxtLink>
              </Button>
            </div>
          </CardHeader>
          <CardContent class="px-0">
            <ReleaseTable :releases="workspace.releases.value.slice(0, 5)" />
          </CardContent>
        </Card>
      </div>
      <p class="text-xs leading-relaxed text-muted-foreground">
        Metrics cover client-reported events retained by this server. Reporting installs are
        distinct IDs in this window, not total active devices.
      </p>
    </template>
  </div>
</template>
