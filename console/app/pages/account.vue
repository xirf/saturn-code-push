<script setup lang="ts">
import AccessManagement from '~/features/access/components/AccessManagement.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
definePageMeta({
  middleware: 'auth',
  title: 'Account & security',
  description: 'Your account information and password.',
})
const identity = useIdentity()
</script>
<template>
  <div class="mx-auto w-full max-w-5xl space-y-5">
    <Card>
      <CardHeader><CardTitle>Your account</CardTitle></CardHeader>
      <CardContent>
        <dl class="divide-y text-sm">
          <div class="flex justify-between gap-4 py-3">
            <dt class="text-muted-foreground">Username</dt>
            <dd>{{ identity.session.data.value?.username }}</dd>
          </div>
          <div class="flex justify-between gap-4 py-3">
            <dt class="text-muted-foreground">Role</dt>
            <dd>
              <Badge variant="secondary">{{ identity.session.data.value?.role }}</Badge>
            </dd>
          </div>
          <div class="flex justify-between gap-4 py-3">
            <dt class="text-muted-foreground">Session lifetime</dt>
            <dd>8 hours</dd>
          </div>
        </dl>
      </CardContent>
    </Card>
    <AccessManagement section="password" />
  </div>
</template>
