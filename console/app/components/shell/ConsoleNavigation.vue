<script setup lang="ts">
import {
  Activity,
  BookOpen,
  GitBranch,
  Layers,
  LayoutDashboard,
  LayoutGrid,
  Settings,
} from '@lucide/vue'
import AppAvatar from '~/components/shared/AppAvatar.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { usePreferencesStore } from '~/stores/preferences'

const preferences = usePreferencesStore()
const workspace = useWorkspace()
const route = useRoute()
const application = workspace.application

// The sidebar is scoped: inside /apps/:appId it shows only that application's surfaces.
const items = computed(() =>
  application.value
    ? [
        { to: workspace.path(), label: 'Overview', icon: LayoutDashboard, exact: true },
        { to: workspace.path('releases'), label: 'Releases', icon: Layers },
        { to: workspace.path('patches'), label: 'Patches', icon: GitBranch },
        { to: workspace.path('analytics'), label: 'Analytics', icon: Activity },
        { to: workspace.path('settings'), label: 'Settings', icon: Settings },
      ]
    : [
        { to: '/', label: 'Applications', icon: LayoutGrid, exact: true },
        { to: '/settings', label: 'Server settings', icon: Settings },
      ],
)

function active(item: { to: string; exact?: boolean }) {
  if (item.exact) return route.path === item.to || (item.to === '/' && route.path === '/new')
  return route.path === item.to || route.path.startsWith(`${item.to}/`)
}
function close() {
  preferences.mobileNavigationOpen = false
}
</script>

<template>
  <div class="flex h-full flex-col bg-sidebar px-3 py-4 text-sidebar-foreground">
    <NuxtLink
      v-if="application"
      :to="workspace.path()"
      class="mb-4 flex items-center gap-3 rounded-lg border bg-card/60 p-2.5 transition-colors hover:bg-sidebar-accent"
      @click="close"
    >
      <AppAvatar
        :name="application.display_name"
        :seed="application.id"
      />
      <span class="min-w-0">
        <span class="block truncate text-sm font-medium text-foreground">
          {{ application.display_name }}
        </span>
        <span class="block truncate font-mono text-[11px] text-muted-foreground">
          {{ application.id.slice(0, 8) }}
        </span>
      </span>
    </NuxtLink>
    <p
      v-else
      class="mb-2 px-3 text-xs font-medium tracking-wide text-muted-foreground uppercase"
    >
      Workspace
    </p>
    <nav
      aria-label="Main navigation"
      class="flex-1 space-y-1"
    >
      <NuxtLink
        v-for="item in items"
        :key="item.to"
        :to="item.to"
        :aria-current="active(item) ? 'page' : undefined"
        :class="[
          'flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors',
          active(item)
            ? 'bg-sidebar-accent font-medium text-sidebar-accent-foreground'
            : 'text-muted-foreground hover:bg-sidebar-accent hover:text-foreground',
        ]"
        @click="close"
      >
        <component
          :is="item.icon"
          class="size-4 shrink-0"
        />
        {{ item.label }}
      </NuxtLink>
    </nav>
    <NuxtLink
      to="/docs"
      class="flex items-center gap-3 rounded-md px-3 py-2 text-sm text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-foreground"
      @click="close"
    >
      <BookOpen class="size-4 shrink-0" />
      Documentation
    </NuxtLink>
  </div>
</template>
