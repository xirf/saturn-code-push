<script setup lang="ts">
defineProps<{ title: string; description?: string }>()
</script>

<template>
  <header class="flex flex-wrap items-end justify-between gap-4">
    <div class="min-w-0">
      <h1 class="font-heading text-2xl font-semibold tracking-[-0.035em]">{{ title }}</h1>
      <p
        v-if="description"
        class="mt-1.5 text-sm text-muted-foreground"
      >
        {{ description }}
      </p>
    </div>
    <div class="flex flex-wrap items-center gap-3">
      <slot />
    </div>
  </header>
</template>
