<script setup lang="ts">
import { ArrowUpRight, LayoutGrid, List, Plus, Rocket, Search } from '@lucide/vue'
import { InputGroup, InputGroupAddon, InputGroupInput } from '~/components/ui/input-group'
import AppAvatar from '~/components/shared/AppAvatar.vue'
import EmptyState from '~/components/shared/EmptyState.vue'
import PageHeader from '~/components/shared/PageHeader.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { formatDate } from '~/lib/format'

definePageMeta({
  middleware: 'auth',
  title: 'Applications',
  description: 'Every application served by this Saturn server.',
})
const workspace = useWorkspace()
const identity = useIdentity()
const isAdmin = computed(() => identity.session.data.value?.role === 'admin')
const search = ref('')
const view = ref<'grid' | 'list'>('grid')

const summaries = computed(() => {
  const catalog = workspace.query.data.value
  return workspace.applications.value
    .filter((app) => app.display_name.toLowerCase().includes(search.value.trim().toLowerCase()))
    .map((app) => {
      const releases = (catalog?.releases ?? []).filter((release) => release.app_id === app.id)
      const releaseIds = new Set(releases.map((release) => release.id))
      const patches = (catalog?.patches ?? []).filter((patch) => releaseIds.has(patch.release_id))
      const latest = releases.at(-1)
      const live = patches.filter((patch) => !patch.is_rolled_back && patch.channel_ids.length)
      return {
        app,
        latest,
        releaseCount: releases.length,
        patchCount: patches.length,
        liveCount: live.length,
        rolledBack: patches.filter((patch) => patch.is_rolled_back).length,
      }
    })
})
const total = computed(() => workspace.applications.value.length)
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      title="Applications"
      description="Pick an application to manage its releases, patches, and access."
    >
      <Button
        v-if="isAdmin"
        as-child
      >
        <NuxtLink to="/new">
          <Plus class="size-4" />
          New application
        </NuxtLink>
      </Button>
    </PageHeader>

    <Card v-if="!total">
      <CardContent class="flex flex-col items-center gap-4 px-6 py-16 text-center">
        <div class="rounded-2xl border bg-muted p-4"><Rocket class="size-8" /></div>
        <div class="space-y-1.5">
          <h2 class="font-heading text-xl font-semibold tracking-tight">
            Create your first application
          </h2>
          <p class="mx-auto max-w-md text-sm leading-relaxed text-muted-foreground">
            An application is one Flutter app and its releases, patches, channels, API keys, and
            device analytics. Everything in Saturn lives inside one.
          </p>
        </div>
        <Button
          v-if="isAdmin"
          as-child
        >
          <NuxtLink to="/new">
            <Plus class="size-4" />
            New application
          </NuxtLink>
        </Button>
        <p
          v-else
          class="text-sm text-muted-foreground"
        >
          Ask a server administrator to create an application for you.
        </p>
      </CardContent>
    </Card>

    <template v-else>
      <div class="flex flex-wrap items-center gap-3">
        <InputGroup class="w-full max-w-md flex-1">
          <InputGroupInput
            v-model="search"
            class="pl-9"
            aria-label="Search applications"
            placeholder="Search applications…"
          />
          <InputGroupAddon><Search aria-hidden="true" /></InputGroupAddon>
        </InputGroup>
        <div
          class="ml-auto flex rounded-md border p-0.5"
          role="group"
          aria-label="Layout"
        >
          <Button
            :variant="view === 'grid' ? 'secondary' : 'ghost'"
            size="icon-sm"
            aria-label="Grid view"
            :aria-pressed="view === 'grid'"
            @click="view = 'grid'"
          >
            <LayoutGrid class="size-4" />
          </Button>
          <Button
            :variant="view === 'list' ? 'secondary' : 'ghost'"
            size="icon-sm"
            aria-label="List view"
            :aria-pressed="view === 'list'"
            @click="view = 'list'"
          >
            <List class="size-4" />
          </Button>
        </div>
      </div>

      <ul
        v-if="summaries.length && view === 'grid'"
        class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3"
        aria-label="Applications"
      >
        <li
          v-for="{ app, latest, releaseCount, patchCount, liveCount, rolledBack } in summaries"
          :key="app.id"
        >
          <NuxtLink
            :to="`/apps/${encodeURIComponent(app.id)}`"
            class="group flex h-full flex-col gap-5 rounded-xl border bg-card p-5 transition-colors hover:border-foreground/30 hover:bg-accent/40 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
          >
            <div class="flex items-start gap-3">
              <AppAvatar
                :name="app.display_name"
                :seed="app.id"
                size="lg"
              />
              <div class="min-w-0 flex-1">
                <h2 class="truncate font-heading text-base font-semibold">
                  {{ app.display_name }}
                </h2>
                <p class="mt-0.5 truncate font-mono text-xs text-muted-foreground">
                  {{ app.id }}
                </p>
              </div>
              <ArrowUpRight
                class="size-4 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100"
              />
            </div>
            <p class="text-sm">
              <template v-if="latest">
                <span class="font-mono">{{ latest.version }}</span>
                <span class="text-muted-foreground">· latest release</span>
              </template>
              <span
                v-else
                class="text-muted-foreground"
              >
                No releases yet
              </span>
            </p>
            <div class="mt-auto flex flex-wrap items-center gap-x-4 gap-y-2 text-xs">
              <span class="text-muted-foreground">
                <span class="font-medium text-foreground tabular-nums">{{ releaseCount }}</span>
                {{ releaseCount === 1 ? 'release' : 'releases' }}
              </span>
              <span class="text-muted-foreground">
                <span class="font-medium text-foreground tabular-nums">{{ patchCount }}</span>
                {{ patchCount === 1 ? 'patch' : 'patches' }}
              </span>
              <Badge
                v-if="liveCount"
                variant="secondary"
              >
                <span class="size-1.5 rounded-full bg-emerald-400" />
                {{ liveCount }} live
              </Badge>
              <Badge
                v-if="rolledBack"
                variant="outline"
              >
                {{ rolledBack }} rolled back
              </Badge>
            </div>
          </NuxtLink>
        </li>
      </ul>

      <Card
        v-else-if="summaries.length"
        class="py-0"
      >
        <ul
          class="divide-y"
          aria-label="Applications"
        >
          <li
            v-for="{ app, latest, releaseCount, patchCount, liveCount } in summaries"
            :key="app.id"
          >
            <NuxtLink
              :to="`/apps/${encodeURIComponent(app.id)}`"
              class="flex flex-wrap items-center gap-x-6 gap-y-2 px-5 py-4 transition-colors hover:bg-accent/40 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none focus-visible:ring-inset"
            >
              <AppAvatar
                :name="app.display_name"
                :seed="app.id"
              />
              <span class="min-w-0 flex-1 basis-48">
                <span class="block truncate text-sm font-medium">{{ app.display_name }}</span>
                <span class="block truncate font-mono text-xs text-muted-foreground">
                  {{ app.id }}
                </span>
              </span>
              <span class="w-28 font-mono text-xs text-muted-foreground">
                {{ latest?.version ?? 'No releases' }}
              </span>
              <span class="w-24 text-xs text-muted-foreground tabular-nums">
                {{ releaseCount }} rel · {{ patchCount }} patch
              </span>
              <span class="w-20">
                <Badge
                  v-if="liveCount"
                  variant="secondary"
                >
                  {{ liveCount }} live
                </Badge>
              </span>
              <span class="w-24 text-xs text-muted-foreground">
                {{ formatDate(app.created_at) }}
              </span>
            </NuxtLink>
          </li>
        </ul>
      </Card>

      <Card v-else>
        <EmptyState
          title="No matching applications"
          :description="`Nothing matches “${search}”. Check the spelling or clear the search.`"
        />
      </Card>
    </template>
  </div>
</template>
