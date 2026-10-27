<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from 'cn'

const props = withDefaults(defineProps<{
  align?: 'inline-start' | 'inline-end'
  class?: HTMLAttributes['class']
}>(), { align: 'inline-start' })
</script>

<template>
  <div
    data-slot="input-group-addon"
    :data-align="props.align"
    :class="cn('pointer-events-none absolute inset-y-0 flex items-center justify-center text-muted-foreground [&_svg]:size-4 [&_svg]:shrink-0', props.align === 'inline-start' ? 'left-0 pl-3' : 'right-0 pr-3', props.class)"
  >
    <slot />
  </div>
</template>
