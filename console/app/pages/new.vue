<script setup lang="ts">
import { Check, LoaderCircle } from '@lucide/vue'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import PageHeader from '~/components/shared/PageHeader.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { useCreateApplication } from '~/features/applications/queries/create-application'
import { createApplicationSchema } from '~/features/applications/types/application'
import { errorMessage } from '~/lib/api/client'

definePageMeta({
  middleware: 'auth',
  title: 'New application',
  description: 'Create an application, then publish its first release from your CLI.',
})
const identity = useIdentity()
const scope = useSessionScope()
const { mutateAsync, asyncStatus } = useCreateApplication()
const isAdmin = computed(() => identity.session.data.value?.role === 'admin')
const name = ref('')
const error = ref('')
const busy = computed(() => asyncStatus.value === 'loading')
const steps = [
  'A release pipeline with stable, beta, and staging delivery channels',
  'An application ID for shorebird.yaml and your CI',
  'Dedicated API keys, GitHub connection, and device analytics',
]

async function submit() {
  error.value = ''
  const parsed = createApplicationSchema.safeParse({ display_name: name.value })
  if (!parsed.success) {
    error.value = parsed.error.issues[0]?.message ?? 'Check the application name'
    return
  }
  try {
    const { app } = await mutateAsync({ ...parsed.data, generation: scope.generation })
    await navigateTo(`/apps/${encodeURIComponent(app.id)}`)
  } catch (cause) {
    error.value = errorMessage(cause)
  }
}
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <PageHeader
      title="Create a new application"
      description="Each application has its own releases, patches, keys, and analytics."
    />
    <Card v-if="isAdmin">
      <form @submit.prevent="submit">
        <CardContent class="grid gap-8 md:grid-cols-[1fr_16rem]">
          <div class="space-y-5">
            <div class="space-y-2">
              <Label for="application-name">Application name</Label>
              <Input
                id="application-name"
                v-model="name"
                placeholder="Customer mobile app"
                required
                autofocus
                :disabled="busy"
              />
              <p class="text-xs text-muted-foreground">
                Shown in the console. You can use the name of your store listing.
              </p>
            </div>
            <ErrorNotice
              v-if="error"
              :message="error"
            />
          </div>
          <div class="space-y-3 rounded-lg border bg-muted/30 p-4">
            <p class="text-sm font-medium">What you get</p>
            <ul class="space-y-2.5 text-xs leading-relaxed text-muted-foreground">
              <li
                v-for="step in steps"
                :key="step"
                class="flex gap-2"
              >
                <Check class="mt-0.5 size-3.5 shrink-0 text-emerald-400" />
                {{ step }}
              </li>
            </ul>
          </div>
        </CardContent>
        <CardFooter class="justify-between gap-3 border-t pt-6">
          <Button
            type="button"
            variant="ghost"
            as-child
            :disabled="busy"
          >
            <NuxtLink to="/">Cancel</NuxtLink>
          </Button>
          <Button
            type="submit"
            :disabled="busy"
          >
            <LoaderCircle
              v-if="busy"
              class="size-4 animate-spin"
            />
            Create application
          </Button>
        </CardFooter>
      </form>
    </Card>
    <Card v-else>
      <CardContent
        role="alert"
        class="text-sm text-muted-foreground"
      >
        Only administrators can create applications.
      </CardContent>
    </Card>
  </div>
</template>
