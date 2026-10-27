<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from 'cn'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
const model = defineModel<string>()
</script>

<template>
  <input
    v-model="model"
    data-slot="input-group-control"
    :class="cn('h-full w-full min-w-0 rounded-lg bg-transparent px-3 py-1 text-sm outline-none placeholder:text-muted-foreground disabled:pointer-events-none disabled:opacity-50', props.class)"
  />
</template>
