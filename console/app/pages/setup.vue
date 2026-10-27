<script setup lang="ts">
import { LoaderCircle, ShieldCheck } from '@lucide/vue'
import SaturnMark from '~/components/brand/SaturnMark.vue'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useSetupQuery } from '~/features/identity/queries/setup'
import { setupSchema } from '~/features/identity/types/session'
import { errorMessage } from '~/lib/api/client'

definePageMeta({ layout: false })
useHead({ title: 'Set up Saturn' })
const identity = useIdentity()
const setup = useSetupQuery()
const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const token = ref('')
const busy = ref(false)
const error = ref('')
watch(
  [setup.data, identity.session.data],
  ([status, user]) => {
    if (user) void navigateTo('/')
    else if (status && !status.required) void navigateTo('/login')
  },
  { immediate: true },
)
onBeforeUnmount(() => {
  password.value = ''
  confirmPassword.value = ''
  token.value = ''
})
async function submit() {
  if (busy.value) return
  error.value = ''
  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords must match'
    return
  }
  const parsed = setupSchema.safeParse({
    username: username.value,
    password: password.value,
    setup_token: token.value,
  })
  if (!parsed.success) {
    error.value = parsed.error.issues[0]?.message ?? 'Check your details'
    return
  }
  busy.value = true
  try {
    await identity.completeSetup(parsed.data)
  } catch (cause) {
    error.value = errorMessage(cause)
    await setup.refresh()
  } finally {
    busy.value = false
    password.value = ''
    confirmPassword.value = ''
    token.value = ''
  }
}
</script>

<template>
  <main class="flex min-h-svh items-center justify-center bg-background px-6 py-12">
    <div class="w-full max-w-md space-y-6">
      <div class="flex items-center gap-3 text-xl font-semibold">
        <SaturnMark class="size-9" />
        saturn
      </div>
      <Card>
        <CardHeader>
          <h1 class="font-heading text-2xl font-semibold">Set up your workspace</h1>
          <CardDescription>
            Create the first administrator account. You can create accounts for your team in
            Settings afterward.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-5">
          <ErrorNotice
            v-if="error || setup.error.value"
            :message="error || errorMessage(setup.error.value)"
          />
          <p
            v-if="!setup.data.value"
            class="text-sm text-muted-foreground"
            role="status"
          >
            Checking workspace setup…
          </p>
          <template v-else-if="!setup.data.value.enabled">
            <p class="text-sm">
              Set a random
              <code>SETUP_TOKEN</code>
              of 32–256 characters in your deployment's environment settings, then restart Saturn.
            </p>
            <p class="text-sm text-muted-foreground">
              You can do this through your hosting dashboard or Compose environment file. No
              container shell is needed.
            </p>
            <Button
              variant="outline"
              :disabled="setup.asyncStatus.value === 'loading'"
              @click="setup.refresh()"
            >
              Check again
            </Button>
          </template>
          <form
            v-else
            class="space-y-4"
            @submit.prevent="submit"
          >
            <div class="space-y-2">
              <Label for="setup-token">Setup token</Label>
              <Input
                id="setup-token"
                v-model="token"
                type="password"
                autocomplete="off"
                required
                maxlength="256"
                :disabled="busy"
              />
              <p class="text-xs text-muted-foreground">
                Use the token you set in deployment settings. It is only used to create the first
                account.
              </p>
            </div>
            <div class="space-y-2">
              <Label for="setup-username">Username</Label>
              <Input
                id="setup-username"
                v-model="username"
                autocomplete="username"
                required
                maxlength="128"
                :disabled="busy"
              />
            </div>
            <div class="space-y-2">
              <Label for="setup-password">Password</Label>
              <Input
                id="setup-password"
                v-model="password"
                type="password"
                autocomplete="new-password"
                required
                minlength="12"
                maxlength="1024"
                :disabled="busy"
              />
            </div>
            <div class="space-y-2">
              <Label for="setup-confirm-password">Confirm password</Label>
              <Input
                id="setup-confirm-password"
                v-model="confirmPassword"
                type="password"
                autocomplete="new-password"
                required
                minlength="12"
                maxlength="1024"
                :disabled="busy"
              />
            </div>
            <Button
              type="submit"
              class="w-full"
              :disabled="busy"
            >
              <LoaderCircle
                v-if="busy"
                class="size-4 animate-spin"
              />
              {{ busy ? 'Creating workspace…' : 'Create administrator' }}
            </Button>
          </form>
          <p class="flex gap-2 text-xs text-muted-foreground">
            <ShieldCheck class="size-4 shrink-0" />
            First-account setup closes after completion. Team access and API keys are managed in
            Settings.
          </p>
        </CardContent>
      </Card>
      <NuxtLink
        to="/docs/accounts-and-postgresql"
        class="block text-center text-sm text-muted-foreground underline"
      >
        Deployment and account setup guide
      </NuxtLink>
    </div>
  </main>
</template>
