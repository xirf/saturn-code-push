<script setup lang="ts">
import { GitBranch, KeyRound, SlidersHorizontal } from '@lucide/vue'
import SettingsShell from '~/components/shell/SettingsShell.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'

definePageMeta({ middleware: 'auth' })
const workspace = useWorkspace()
const sections = computed(() => [
  {
    label: 'Application',
    links: [
      { to: workspace.path('settings'), label: 'General', icon: SlidersHorizontal },
      { to: workspace.path('settings/api-keys'), label: 'API keys', icon: KeyRound },
    ],
  },
  {
    label: 'Integrations',
    links: [{ to: workspace.path('settings/github'), label: 'GitHub', icon: GitBranch }],
  },
])
</script>

<template>
  <SettingsShell
    title="Settings"
    :sections="sections"
  >
    <NuxtPage />
  </SettingsShell>
</template>
