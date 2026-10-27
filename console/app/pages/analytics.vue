<script setup lang="ts">
import EventStats from '~/features/telemetry/components/EventStats.vue'
import ActivityChart from '~/features/telemetry/components/ActivityChart.vue'
import EventLog from '~/features/telemetry/components/EventLog.vue'

definePageMeta({
  middleware: 'auth',
  title: 'Analytics',
  description: 'Downloads, launches, and failures reported by devices.',
})
</script>

<template>
  <div class="space-y-6">
    <EventStats />
    <ActivityChart />
    <EventLog />
  </div>
</template>
