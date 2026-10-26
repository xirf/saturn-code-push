<script setup lang="ts">
import 'vue-sonner/style.css'
</script>

<template>
  <TooltipProvider>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <Toaster
      class="pointer-events-auto"
      theme="dark"
      position="bottom-right"
      close-button
    />
  </TooltipProvider>
</template>
