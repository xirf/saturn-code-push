<script setup lang="ts">
import { ArrowUpRight, Layers } from '@lucide/vue'
import EventStats from '~/features/telemetry/components/EventStats.vue'
import ActivityChart from '~/features/telemetry/components/ActivityChart.vue'
import ReleaseTable from '~/features/distribution/components/ReleaseTable.vue'
import EmptyState from '~/components/shared/EmptyState.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'

definePageMeta({
  middleware: 'auth',
  title: 'Overview',
  description: 'Release activity for the selected application.',
})
const workspace = useWorkspace()
const latestRelease = computed(() => workspace.releases.value[0])
const latestPatchCount = computed(
  () =>
    workspace.patches.value.filter((patch) => patch.release_id === latestRelease.value?.id).length,
)
</script>

<template>
  <div class="space-y-6">
    <EventStats />
    <div class="grid gap-6 xl:grid-cols-[1fr_20rem]">
      <ActivityChart />
      <Card class="gap-5">
        <CardHeader>
          <CardDescription>Latest baseline</CardDescription>
          <CardTitle class="font-heading text-xl">
            {{ latestRelease?.version ?? 'No release yet' }}
          </CardTitle>
        </CardHeader>
        <CardContent
          v-if="latestRelease"
          class="space-y-5"
        >
          <div class="flex flex-wrap gap-2">
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
          </div>
          <dl class="space-y-3 text-xs">
            <div class="flex items-center justify-between gap-3">
              <dt class="text-muted-foreground">Flutter revision</dt>
              <dd
                class="max-w-40 truncate font-mono"
                :title="latestRelease.flutter_revision"
              >
                {{ latestRelease.flutter_revision }}
              </dd>
            </div>
            <div class="flex items-center justify-between">
              <dt class="text-muted-foreground">Patches</dt>
              <dd>{{ latestPatchCount }}</dd>
            </div>
            <div class="flex items-center justify-between">
              <dt class="text-muted-foreground">Delivery channels</dt>
              <dd>{{ workspace.channels.value.length }}</dd>
            </div>
          </dl>
        </CardContent>
        <CardContent
          v-else
          class="space-y-3 text-sm text-muted-foreground"
        >
          <Layers class="size-6" />
          <p>Publish a baseline with the Saturn CLI to begin delivering patches.</p>
        </CardContent>
        <CardFooter class="mt-auto">
          <Button
            variant="outline"
            class="w-full"
            as-child
          >
            <NuxtLink :to="latestRelease ? `/releases/${latestRelease.id}` : '/releases'">
              View release
              <ArrowUpRight class="ml-auto size-4" />
            </NuxtLink>
          </Button>
        </CardFooter>
      </Card>
    </div>
    <Card>
      <CardHeader>
        <div class="flex items-center justify-between gap-4">
          <div>
            <CardTitle>Recent releases</CardTitle>
            <CardDescription class="mt-1">
              The five most recent baselines for this application.
            </CardDescription>
          </div>
          <Button
            variant="ghost"
            size="sm"
            as-child
          >
            <NuxtLink to="/releases">
              View all
              <ArrowUpRight class="size-4" />
            </NuxtLink>
          </Button>
        </div>
      </CardHeader>
      <CardContent class="px-0">
        <ReleaseTable
          v-if="workspace.application.value"
          :releases="workspace.releases.value.slice(0, 5)"
        />
        <EmptyState
          v-else
          title="Your workspace starts here"
          description="Create an application, then publish its baseline release with the CLI."
        />
      </CardContent>
    </Card>
    <p class="text-xs leading-relaxed text-muted-foreground">
      Metrics cover client-reported events retained by this server. Reporting installs are distinct
      IDs in this window, not total active devices.
    </p>
  </div>
</template>
