<script setup lang="ts">
import { LoaderCircle } from '@lucide/vue'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import { errorMessage } from '~/lib/api/client'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { usePatchAction } from '../queries/patch-action'
import type { Patch, PatchAction } from '../types/patch'

const props = defineProps<{ patch: Patch; action: PatchAction }>()
const open = defineModel<boolean>('open', { default: false })
const channelId = ref('')
const error = ref('')
const workspace = useWorkspace()
const scope = useSessionScope()
const { mutateAsync, asyncStatus } = usePatchAction()
const busy = computed(() => asyncStatus.value === 'loading')
const label = computed(
  () => ({ promote: 'Promote', rollback: 'Roll back', restore: 'Restore' })[props.action],
)
const description = computed(
  () =>
    ({
      promote: 'Make this patch available to devices on the selected channel.',
      rollback:
        'Devices will remove this patch on their next update check and use a prior patch or baseline after restart.',
      restore: 'Make this patch available again on its existing channels.',
    })[props.action],
)

watch(open, (value) => {
  if (!value) return
  error.value = ''
  channelId.value = String(
    workspace.channels.value.find((channel) => channel.name === 'stable')?.id ??
      workspace.channels.value[0]?.id ??
      '',
  )
})

async function submit() {
  error.value = ''
  const appId = workspace.query.data.value?.releases.find(
    (release) => release.id === props.patch.release_id,
  )?.app_id
  if (!appId) {
    error.value = 'The patch release is unavailable. Refresh the workspace.'
    return
  }
  try {
    await mutateAsync({
      appId,
      patch: props.patch,
      action: props.action,
      channelId: Number(channelId.value),
      generation: scope.generation,
    })
    open.value = false
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
        <DialogTitle>{{ label }} patch #{{ patch.number }}</DialogTitle>
        <DialogDescription>{{ description }}</DialogDescription>
      </DialogHeader>
      <form
        class="space-y-5"
        @submit.prevent="submit"
      >
        <div
          v-if="action === 'promote'"
          class="space-y-2"
        >
          <Label for="patch-channel">Delivery channel</Label>
          <Select
            v-model="channelId"
            :disabled="busy"
          >
            <SelectTrigger
              id="patch-channel"
              class="w-full"
            >
              <SelectValue placeholder="Select a channel" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="channel in workspace.channels.value"
                :key="channel.id"
                :value="String(channel.id)"
              >
                {{ channel.name }}
              </SelectItem>
            </SelectContent>
          </Select>
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
            :variant="action === 'rollback' ? 'destructive' : 'default'"
            :disabled="busy || (action === 'promote' && !channelId)"
          >
            <LoaderCircle
              v-if="busy"
              class="size-4 animate-spin"
            />
            {{ label }} patch
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
