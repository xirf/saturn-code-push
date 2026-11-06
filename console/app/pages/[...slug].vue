<script setup lang="ts">
import EmptyState from '~/components/shared/EmptyState.vue'

definePageMeta({ middleware: 'auth', title: 'Not found' })
</script>

<template>
  <Card>
    <EmptyState
      title="Page not found"
      description="This address does not exist in the console."
    >
      <Button
        variant="outline"
        as-child
      >
        <NuxtLink to="/">View all applications</NuxtLink>
      </Button>
    </EmptyState>
  </Card>
</template>
