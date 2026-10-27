<script setup lang="ts">
import { Check, Copy } from '@lucide/vue'
import { toast } from 'vue-sonner'
import { ScrollArea, ScrollBar } from '~/components/ui/scroll-area'

const props = withDefaults(
  defineProps<{
    code: string
    title?: string
    wrap?: boolean
  }>(),
  { title: '', wrap: false },
)
const copied = ref(false)
let copiedTimer: ReturnType<typeof setTimeout> | undefined
onBeforeUnmount(() => clearTimeout(copiedTimer))

async function copy() {
  try {
    await navigator.clipboard.writeText(props.code)
    copied.value = true
    clearTimeout(copiedTimer)
    copiedTimer = setTimeout(() => (copied.value = false), 1500)
  } catch {
    toast.error('Clipboard unavailable. Select and copy the code manually.')
  }
}
</script>

<template>
  <section
    class="min-w-0 overflow-hidden rounded-lg border bg-muted/30"
    :aria-label="title || 'Code block'"
  >
    <header class="flex min-h-11 items-center justify-between gap-3 border-b bg-muted/50 px-3">
      <p class="min-w-0 truncate font-mono text-xs text-muted-foreground">
        {{ title || 'Code' }}
      </p>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        class="h-8 shrink-0 gap-2 px-2 text-xs"
        :aria-label="copied ? 'Copied code' : 'Copy code'"
        @click="copy"
      >
        <Check
          v-if="copied"
          class="size-3.5"
          aria-hidden="true"
        />
        <Copy
          v-else
          class="size-3.5"
          aria-hidden="true"
        />
        {{ copied ? 'Copied' : 'Copy' }}
      </Button>
    </header>
    <ScrollArea class="w-full">
      <pre
        tabindex="0"
        dir="ltr"
        class="p-4 font-mono text-xs leading-relaxed outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
        :class="wrap ? 'w-full whitespace-pre-wrap break-words' : 'w-max min-w-full whitespace-pre'"
      ><code>{{ code }}</code></pre>
      <ScrollBar orientation="horizontal" />
    </ScrollArea>
  </section>
</template>
