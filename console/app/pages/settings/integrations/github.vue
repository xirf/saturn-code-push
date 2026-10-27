<script setup lang="ts">
import GitHubIntegration from '~/features/integrations/components/GitHubIntegration.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
definePageMeta({
  title: 'GitHub',
  description: 'Manage repository connections and deployment sync.',
  settingsGroup: 'Connections',
})
const identity = useIdentity()
</script>
<template>
  <GitHubIntegration v-if="identity.session.data.value?.role === 'admin'" />
  <p
    v-else
    role="alert"
    class="text-sm text-muted-foreground"
  >
    Only administrators can view these settings.
  </p>
</template>
