<script setup lang="ts">
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent, AriaComponent } from 'echarts/components'
import { SVGRenderer } from 'echarts/renderers'
import VChart from 'vue-echarts'
import { useTelemetry } from '../composables/useTelemetry'

use([LineChart, GridComponent, TooltipComponent, LegendComponent, AriaComponent, SVGRenderer])
const { summary } = useTelemetry()
const options = computed(() => ({
  animation: false,
  aria: {
    enabled: true,
    description: 'Patch downloads and successful first launches per UTC day.',
  },
  color: ['#d4d4d4', '#60a5fa'],
  textStyle: { fontFamily: 'sans-serif' },
  tooltip: { trigger: 'axis', renderMode: 'richText' },
  legend: { textStyle: { color: '#a3a3a3' }, top: 0, right: 0 },
  grid: { top: 45, left: 40, right: 16, bottom: 25 },
  xAxis: {
    type: 'category',
    data: summary.value.chart.map((day) => day.day.slice(5)),
    axisLabel: { color: '#a3a3a3' },
    axisLine: { lineStyle: { color: '#404040' } },
  },
  yAxis: {
    type: 'value',
    minInterval: 1,
    axisLabel: { color: '#a3a3a3' },
    splitLine: { lineStyle: { color: '#303030', type: 'dashed' } },
  },
  series: [
    {
      name: 'Downloads',
      type: 'line',
      smooth: true,
      showSymbol: false,
      areaStyle: { opacity: 0.08 },
      data: summary.value.chart.map((day) => day.downloads),
    },
    {
      name: 'Launches',
      type: 'line',
      smooth: true,
      showSymbol: false,
      data: summary.value.chart.map((day) => day.launches),
    },
  ],
}))
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Patch activity</CardTitle>
      <CardDescription>Downloads and successful first launches, by UTC day.</CardDescription>
    </CardHeader>
    <CardContent>
      <div
        class="h-64 w-full"
        data-testid="activity-chart"
      >
        <VChart
          class="h-full w-full"
          :option="options"
          :init-options="{ renderer: 'svg' }"
          autoresize
        />
      </div>
    </CardContent>
  </Card>
</template>
