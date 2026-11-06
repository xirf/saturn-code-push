<script setup lang="ts">
import AccessManagement from '~/features/access/components/AccessManagement.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'

definePageMeta({
  title: 'API keys',
  description: 'Credentials for publishing and deployment reporting. Scoped to this application.',
})
const identity = useIdentity()
</script>

<template>
  <AccessManagement
    v-if="identity.session.data.value?.role === 'admin'"
    section="keys"
  />
  <p
    v-else
    role="alert"
    class="text-sm text-muted-foreground"
  >
    Only administrators can manage API keys.
  </p>
</template>
