<script setup lang="ts">
import { ArrowRight, LoaderCircle, LockKeyhole, ShieldCheck } from '@lucide/vue'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import { errorMessage } from '~/lib/api/client'
import { useIdentity } from '../composables/useIdentity'
import { loginSchema } from '../types/session'

const { login } = useIdentity()
const username = ref('')
const password = ref('')
const error = ref('')
const busy = computed(() => login.asyncStatus.value === 'loading')

async function submit() {
  error.value = ''
  const input = loginSchema.safeParse({ username: username.value, password: password.value })
  if (!input.success) {
    error.value = input.error.issues[0]?.message ?? 'Check your credentials'
    return
  }
  try {
    await login.mutateAsync(input.data)
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    password.value = ''
  }
}
</script>

<template>
  <div class="w-full max-w-sm">
    <div class="mb-7 inline-flex rounded-xl border bg-muted p-3">
      <LockKeyhole class="size-6" />
    </div>
    <h1 class="font-heading text-[28px] font-semibold tracking-tight">Welcome back</h1>
    <p class="mt-2 text-sm text-muted-foreground">Sign in to your release console.</p>
    <form
      class="mt-8 space-y-5"
      @submit.prevent="submit"
    >
      <div class="space-y-2">
        <Label for="username">Username</Label>
        <Input
          id="username"
          v-model="username"
          class="h-11"
          autocomplete="username"
          placeholder="Your username"
          required
          maxlength="128"
          :disabled="busy"
        />
      </div>
      <div class="space-y-2">
        <Label for="password">Password</Label>
        <Input
          id="password"
          v-model="password"
          class="h-11"
          type="password"
          autocomplete="current-password"
          placeholder="Enter your password"
          required
          maxlength="1024"
          :disabled="busy"
        />
      </div>
      <ErrorNotice
        v-if="error"
        :message="error"
      />
      <Button
        class="h-11 w-full"
        type="submit"
        :disabled="busy"
      >
        <LoaderCircle
          v-if="busy"
          class="size-4 animate-spin"
        />
        Sign in
        <ArrowRight class="ml-auto size-4" />
      </Button>
    </form>
    <div class="mt-6 flex items-start gap-2 text-xs leading-relaxed text-muted-foreground">
      <ShieldCheck class="mt-0.5 size-4 shrink-0" />
      <p>
        Protected by your self-hosted Saturn server. Need access? Contact your workspace
        administrator.
      </p>
    </div>
  </div>
</template>
