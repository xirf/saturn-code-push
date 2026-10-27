<script setup lang="ts">
import AccessManagement from '~/features/access/components/AccessManagement.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
definePageMeta({
  title: 'Accounts',
  description: 'Manage user accounts and their access to this server.',
  settingsGroup: 'Management',
})
const identity = useIdentity()
</script>
<template>
  <AccessManagement
    v-if="identity.session.data.value?.role === 'admin'"
    section="accounts"
  />
  <p
    v-else
    role="alert"
    class="text-sm text-muted-foreground"
  >
    Only administrators can manage accounts.
  </p>
</template>
