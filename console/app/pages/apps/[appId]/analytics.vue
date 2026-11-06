<script setup lang="ts">
import PageHeader from '~/components/shared/PageHeader.vue'
import EventStats from '~/features/telemetry/components/EventStats.vue'
import ActivityChart from '~/features/telemetry/components/ActivityChart.vue'
import EventLog from '~/features/telemetry/components/EventLog.vue'
import { usePreferencesStore } from '~/stores/preferences'

definePageMeta({
  middleware: 'auth',
  title: 'Analytics',
  description: 'Downloads, launches, and failures reported by devices.',
})
const preferences = usePreferencesStore()
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      title="Analytics"
      description="Downloads, launches, and failures reported by devices."
    >
      <Select
        :model-value="String(preferences.periodDays)"
        @update:model-value="(value) => (preferences.periodDays = Number(value))"
      >
        <SelectTrigger
          class="w-44 text-sm"
          aria-label="Reporting period"
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="7">Last 7 UTC days</SelectItem>
          <SelectItem value="14">Last 14 UTC days</SelectItem>
          <SelectItem value="30">Last 30 UTC days</SelectItem>
        </SelectContent>
      </Select>
    </PageHeader>
    <EventStats />
    <ActivityChart />
    <EventLog />
  </div>
</template>
