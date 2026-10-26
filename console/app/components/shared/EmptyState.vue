<script setup lang="ts">
import { PackageOpen } from '@lucide/vue'

defineProps<{ title: string; description: string }>()
</script>

<template>
  <div class="flex flex-col items-center gap-3 px-6 py-14 text-center">
    <div class="rounded-xl border bg-muted p-3">
      <PackageOpen class="size-6 text-muted-foreground" />
    </div>
    <h3 class="font-semibold">{{ title }}</h3>
    <p class="max-w-sm text-sm leading-relaxed text-muted-foreground">{{ description }}</p>
    <slot />
  </div>
</template>
