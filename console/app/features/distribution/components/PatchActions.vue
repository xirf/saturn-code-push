<script setup lang="ts">
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { ArrowUpRight, MoreHorizontal, RotateCcw, Undo2 } from '@lucide/vue'
import PatchActionDialog from './PatchActionDialog.vue'
import type { Patch, PatchAction } from '../types/patch'

defineProps<{ patch: Patch }>()

const identity = useIdentity()
const canPublish = computed(() =>
  ['admin', 'publisher'].includes(identity.session.data.value?.role ?? ''),
)
const menuOpen = ref(false)
const dialogOpen = ref(false)
const action = ref<PatchAction>('promote')

async function chooseAction(value: PatchAction) {
  menuOpen.value = false
  action.value = value
  await nextTick()
  dialogOpen.value = true
}
</script>

<template>
  <div
    v-if="canPublish"
    class="flex justify-end gap-2"
  >
    <Button
      size="sm"
      variant="secondary"
      :disabled="patch.is_rolled_back"
      @click="chooseAction('promote')"
    >
      Promote
      <ArrowUpRight class="size-3.5" />
    </Button>
    <DropdownMenu v-model:open="menuOpen">
      <DropdownMenuTrigger as-child>
        <Button
          variant="ghost"
          size="icon-sm"
          :aria-label="`Actions for patch ${patch.number}`"
        >
          <MoreHorizontal class="size-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        class="min-w-44"
        :side-offset="6"
        @close-auto-focus="
          (event) => {
            if (dialogOpen) event.preventDefault()
          }
        "
      >
        <DropdownMenuItem
          v-if="patch.is_rolled_back"
          @select="chooseAction('restore')"
        >
          <RotateCcw class="size-4" />
          Restore patch
        </DropdownMenuItem>
        <DropdownMenuItem
          v-else
          variant="destructive"
          @select="chooseAction('rollback')"
        >
          <Undo2 class="size-4" />
          Roll back patch
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
    <PatchActionDialog
      v-model:open="dialogOpen"
      :patch="patch"
      :action="action"
    />
  </div>
</template>
