<script setup lang="ts">
import EmptyState from '~/components/shared/EmptyState.vue'
import { useTelemetry } from '../composables/useTelemetry'
import { eventLabel } from '../lib/metrics'

const { summary } = useTelemetry()
const rows = computed(() => summary.value.period.slice(-100).reverse())
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Device event log</CardTitle>
      <CardDescription>Most recent 100 reports within your selected period.</CardDescription>
    </CardHeader>
    <CardContent class="px-0">
      <Table v-if="rows.length">
        <TableHeader>
          <TableRow>
            <TableHead>Event</TableHead>
            <TableHead>Release</TableHead>
            <TableHead>Patch</TableHead>
            <TableHead>Device</TableHead>
            <TableHead>Received</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="(record, index) in rows"
            :key="`${record.received_at}-${index}`"
          >
            <TableCell>
              <span class="font-medium">{{ eventLabel(record.event.type) }}</span>
              <p
                v-if="record.event.message"
                class="mt-1 max-w-xs truncate text-xs text-muted-foreground"
                :title="record.event.message"
              >
                {{ record.event.message }}
              </p>
            </TableCell>
            <TableCell class="font-mono text-xs">
              {{ record.event.release_version ?? '—' }}
            </TableCell>
            <TableCell>
              {{ record.event.patch_number ? `#${record.event.patch_number}` : '—' }}
            </TableCell>
            <TableCell class="text-xs text-muted-foreground">
              {{ record.event.platform ?? '—' }} / {{ record.event.arch ?? '—' }}
            </TableCell>
            <TableCell class="text-xs text-muted-foreground">
              {{ new Date(record.received_at).toLocaleString() }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <EmptyState
        v-else
        title="No reports in this period"
        description="Device events appear after a patch downloads or launches."
      />
    </CardContent>
    <CardFooter class="text-xs leading-relaxed text-muted-foreground">
      Client-reported events within the retained window. Duplicates count again; these are not total
      device or adoption figures.
    </CardFooter>
  </Card>
</template>
