<script setup lang="ts">
import { useTelemetry } from '../composables/useTelemetry'

const { summary } = useTelemetry()
const cards = computed(() => [
  {
    label: 'Patch downloads',
    value: summary.value.downloads,
    caption: 'Download reports',
  },
  {
    label: 'Successful launches',
    value: summary.value.launches,
    caption: 'First-launch reports',
  },
  {
    label: 'Reporting installs',
    value: summary.value.reporting,
    caption: 'Distinct reporting install IDs',
  },
  {
    label: 'Failure reports',
    value: summary.value.failures,
    caption: 'Update and launch failures',
  },
])
</script>

<template>
  <Card class="gap-0 overflow-hidden py-0">
    <CardContent class="grid grid-cols-2 divide-x divide-border p-0 xl:grid-cols-4">
      <div
        v-for="card in cards"
        :key="card.label"
        class="space-y-2 px-5 py-5"
      >
        <p class="text-xs text-muted-foreground">{{ card.label }}</p>
        <p class="font-heading text-2xl font-semibold tabular-nums tracking-tight">
          {{ card.value.toLocaleString() }}
        </p>
        <p class="text-[11px] text-muted-foreground">{{ card.caption }}</p>
      </div>
    </CardContent>
  </Card>
</template>
