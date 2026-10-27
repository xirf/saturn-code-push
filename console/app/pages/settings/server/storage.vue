<script setup lang="ts">
import StorageSettings from '~/features/storage/components/StorageSettings.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
definePageMeta({
  title: 'Artifact storage',
  description: 'Storage configuration for this server.',
  settingsGroup: 'Configuration',
})
const identity = useIdentity()
</script>
<template>
  <StorageSettings v-if="identity.session.data.value?.role === 'admin'" />
  <p
    v-else
    role="alert"
    class="text-sm text-muted-foreground"
  >
    Only administrators can view these settings.
  </p>
</template>
