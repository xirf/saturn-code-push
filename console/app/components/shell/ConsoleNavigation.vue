<script setup lang="ts">
import { Activity, BookOpen, Box, GitBranch, Layers, LayoutDashboard, Settings } from '@lucide/vue'
import SaturnMark from '~/components/brand/SaturnMark.vue'
import { usePreferencesStore } from '~/stores/preferences'

const preferences = usePreferencesStore()
const route = useRoute()
const items = [
  { to: '/', label: 'Overview', icon: LayoutDashboard },
  { to: '/applications', label: 'Applications', icon: Box },
  { to: '/releases', label: 'Releases', icon: Layers },
  { to: '/patches', label: 'Patches', icon: GitBranch },
  { to: '/analytics', label: 'Analytics', icon: Activity },
  { to: '/settings', label: 'Settings', icon: Settings },
  { to: '/docs', label: 'Documentation', icon: BookOpen },
]

function active(path: string) {
  return path === '/' ? route.path === path : route.path.startsWith(path)
}
</script>

<template>
  <div class="flex h-full flex-col bg-sidebar px-3 py-5 text-sidebar-foreground">
    <NuxtLink
      to="/"
      class="mb-9 flex items-center gap-2.5 px-3"
      @click="preferences.mobileNavigationOpen = false"
    >
      <SaturnMark class="size-8 text-foreground" />
      <span class="text-lg font-semibold tracking-tight text-foreground">
        saturn
        <span class="mt-0.5 block text-[9px] font-medium tracking-[0.25em] text-muted-foreground">
          CODE PUSH
        </span>
      </span>
    </NuxtLink>
    <nav
      aria-label="Main navigation"
      class="space-y-1"
    >
      <NuxtLink
        v-for="item in items"
        :key="item.to"
        :to="item.to"
        :aria-current="active(item.to) ? 'page' : undefined"
        :class="[
          'flex items-center gap-3 rounded-md px-3 py-2 text-[13px] transition-colors',
          active(item.to)
            ? 'bg-sidebar-accent font-medium text-sidebar-accent-foreground'
            : 'text-muted-foreground hover:bg-sidebar-accent hover:text-foreground',
        ]"
        @click="preferences.mobileNavigationOpen = false"
      >
        <component
          :is="item.icon"
          class="size-[15px]"
        />
        {{ item.label }}
      </NuxtLink>
    </nav>
  </div>
</template>
