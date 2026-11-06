<script setup lang="ts">
import { AlertCircle } from '@lucide/vue'

defineProps<{ message: string }>()
</script>

<template>
  <Alert
    variant="destructive"
    role="alert"
  >
    <AlertCircle class="size-4" />
    <AlertTitle>Unable to complete the request</AlertTitle>
    <AlertDescription>{{ message }}</AlertDescription>
    <!-- The alert is a two-column grid (icon | text); extra content must stay in the text column. -->
    <div
      v-if="$slots.default"
      class="col-start-2"
    >
      <slot />
    </div>
  </Alert>
</template>
