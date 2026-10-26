<script setup lang="ts">
import { Plus, LoaderCircle } from '@lucide/vue'
import { errorMessage } from '~/lib/api/client'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { useCreateApplication } from '../queries/create-application'
import { createApplicationSchema } from '../types/application'

const open = ref(false)
const name = ref('')
const error = ref('')
const scope = useSessionScope()
const { mutateAsync, asyncStatus } = useCreateApplication()
const busy = computed(() => asyncStatus.value === 'loading')

async function submit() {
  error.value = ''
  const parsed = createApplicationSchema.safeParse({ display_name: name.value })
  if (!parsed.success) {
    error.value = parsed.error.issues[0]?.message ?? 'Check the application name'
    return
  }
  try {
    await mutateAsync({ ...parsed.data, generation: scope.generation })
    open.value = false
    name.value = ''
  } catch (cause) {
    error.value = errorMessage(cause)
  }
}
</script>

<template>
  <Dialog
    :open="open"
    @update:open="
      (value) => {
        if (!busy) open = value
      }
    "
  >
    <DialogTrigger as-child>
      <Button size="sm">
        <Plus class="size-4" />
        New application
      </Button>
    </DialogTrigger>
    <DialogContent
      class="sm:max-w-md"
      :show-close-button="!busy"
      @interact-outside="
        (event) => {
          if (busy) event.preventDefault()
        }
      "
      @escape-key-down="
        (event) => {
          if (busy) event.preventDefault()
        }
      "
    >
      <DialogHeader>
        <DialogTitle>New application</DialogTitle>
        <DialogDescription>Create a dedicated release pipeline for your app.</DialogDescription>
      </DialogHeader>
      <form
        class="space-y-5"
        @submit.prevent="submit"
      >
        <div class="space-y-2">
          <Label for="application-name">Application name</Label>
          <Input
            id="application-name"
            v-model="name"
            placeholder="Customer mobile app"
            required
            :disabled="busy"
          />
        </div>
        <ErrorNotice
          v-if="error"
          :message="error"
        />
        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            :disabled="busy"
            @click="open = false"
          >
            Cancel
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
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
