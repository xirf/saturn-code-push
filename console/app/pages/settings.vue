<script setup lang="ts">
import AccessManagement from '~/features/access/components/AccessManagement.vue'
import { Copy, ShieldCheck, LogOut, Terminal } from '@lucide/vue'
import { toast } from 'vue-sonner'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { errorMessage } from '~/lib/api/client'

definePageMeta({
  middleware: 'auth',
  title: 'Settings',
  description: 'Accounts, API keys, and server access.',
})
const identity = useIdentity()
const origin = ref('')
const error = ref('')
onMounted(() => {
  origin.value = window.location.origin
})

async function copy() {
  try {
    await navigator.clipboard.writeText(origin.value)
    toast.success('API origin copied')
  } catch {
    toast.error('Clipboard unavailable. Select and copy the origin manually.')
  }
}

async function logout() {
  try {
    await identity.logout.mutateAsync()
  } catch (cause) {
    error.value = errorMessage(cause)
  }
}
</script>

<template>
  <div class="space-y-6">
    <ErrorNotice
      v-if="error"
      :message="error"
    />
    <div class="grid gap-6 xl:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Connection</CardTitle>
          <CardDescription>One origin for your console and update API.</CardDescription>
        </CardHeader>
        <CardContent class="space-y-6">
          <div class="flex items-center gap-3">
            <span class="shrink-0 text-sm text-muted-foreground">API origin</span>
            <code
              class="min-w-0 flex-1 truncate text-xs"
              :title="origin"
            >
              {{ origin }}
            </code>
            <Button
              variant="ghost"
              size="icon"
              aria-label="Copy API origin"
              @click="copy"
            >
              <Copy class="size-4" />
            </Button>
          </div>
          <Separator />
          <div class="flex items-start gap-3">
            <Terminal class="mt-1 size-4 shrink-0 text-muted-foreground" />
            <div class="space-y-2">
              <h2 class="text-sm font-medium">Connect your CLI</h2>
              <p class="text-sm leading-relaxed text-muted-foreground">
                Use your server's API key for CLI requests. Dashboard sessions stay in an HTTP-only
                cookie.
              </p>
              <code class="block break-all rounded-md bg-muted p-3 text-xs">
                SHOREBIRD_HOSTED_URL={{ origin }}
              </code>
            </div>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Access & session</CardTitle>
          <CardDescription>Your account session.</CardDescription>
        </CardHeader>
        <CardContent class="space-y-5">
          <div class="flex justify-between text-sm">
            <span class="text-muted-foreground">Username</span>
            <span>{{ identity.session.data.value?.username }}</span>
          </div>
          <div class="flex justify-between text-sm">
            <span class="text-muted-foreground">Session lifetime</span>
            <span>8 hours</span>
          </div>
          <div class="flex justify-between text-sm">
            <span class="text-muted-foreground">Role</span>
            <Badge variant="secondary">{{ identity.session.data.value?.role }}</Badge>
          </div>
          <Separator />
          <div class="flex items-start gap-3">
            <ShieldCheck class="mt-1 size-4 shrink-0 text-muted-foreground" />
            <p class="text-sm leading-relaxed text-muted-foreground">
              Manage your password below. Administrators can manage accounts and CI keys.
            </p>
          </div>
        </CardContent>
        <CardFooter>
          <Button
            variant="outline"
            :disabled="identity.logout.asyncStatus.value === 'loading'"
            @click="logout"
          >
            <LogOut class="size-4" />
            Sign out
          </Button>
        </CardFooter>
      </Card>
    </div>
    <AccessManagement />
  </div>
</template>
