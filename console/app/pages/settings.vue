<script setup lang="ts">
import { Activity, HardDrive, Server, UsersRound } from '@lucide/vue'
import SettingsShell from '~/components/shell/SettingsShell.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'

definePageMeta({ middleware: 'auth', crumb: 'Server settings' })
const identity = useIdentity()
const isAdmin = computed(() => identity.session.data.value?.role === 'admin')
const sections = computed(() => [
  {
    label: 'Server',
    links: [
      { to: '/settings/server/connection', label: 'General', icon: Server },
      ...(isAdmin.value
        ? [{ to: '/settings/server/storage', label: 'Artifact storage', icon: HardDrive }]
        : []),
    ],
  },
  ...(isAdmin.value
    ? [
        {
          label: 'Team',
          links: [{ to: '/settings/management/accounts', label: 'Accounts', icon: UsersRound }],
        },
        {
          label: 'Compliance',
          links: [{ to: '/settings/management/audit', label: 'Audit history', icon: Activity }],
        },
      ]
    : []),
])
</script>

<template>
  <SettingsShell
    title="Server settings"
    :sections="sections"
  >
    <NuxtPage />
  </SettingsShell>
</template>
