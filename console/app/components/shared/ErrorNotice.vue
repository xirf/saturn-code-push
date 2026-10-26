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
    <slot />
  </Alert>
</template>
