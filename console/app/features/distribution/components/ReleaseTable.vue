<script setup lang="ts">
import { ArrowUpRight, Layers } from '@lucide/vue'
import { formatDate } from '~/lib/format'
import EmptyState from '~/components/shared/EmptyState.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { useReleaseRows } from '../composables/useReleaseRows'
import type { Release } from '../types/release'

const props = defineProps<{ releases: Release[] }>()
const workspace = useWorkspace()
const rows = useReleaseRows(toRef(props, 'releases'))
</script>

<template>
  <Table v-if="releases.length">
    <TableHeader>
      <TableRow>
        <TableHead>Release</TableHead>
        <TableHead>Status</TableHead>
        <TableHead>Platforms</TableHead>
        <TableHead>Build</TableHead>
        <TableHead>Created</TableHead>
        <TableHead><span class="sr-only">Details</span></TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow
        v-for="{ release, active, baselineCount, platformNames } in rows"
        :key="release.id"
      >
        <TableCell>
          <NuxtLink
            :to="workspace.path(`releases/${release.id}`)"
            class="flex items-center gap-3 font-medium hover:underline"
          >
            <span class="rounded-md border bg-muted p-2"><Layers class="size-4" /></span>
            <span>
              {{ release.version }}
              <span class="mt-1 block text-xs font-normal text-muted-foreground">
                {{ baselineCount }}
                baseline artifacts
              </span>
            </span>
          </NuxtLink>
        </TableCell>
        <TableCell>
          <Badge :variant="active ? 'secondary' : 'outline'">
            {{ active ? 'Active' : 'Draft' }}
          </Badge>
        </TableCell>
        <TableCell class="text-muted-foreground">
          {{ platformNames }}
        </TableCell>
        <TableCell>
          <code class="text-xs text-muted-foreground">
            {{ release.flutter_revision.slice(0, 10) }}
          </code>
        </TableCell>
        <TableCell class="text-muted-foreground">{{ formatDate(release.created_at) }}</TableCell>
        <TableCell>
          <Button
            variant="ghost"
            size="icon"
            as-child
          >
            <NuxtLink
              :to="workspace.path(`releases/${release.id}`)"
              :aria-label="`View release ${release.version}`"
            >
              <ArrowUpRight class="size-4" />
            </NuxtLink>
          </Button>
        </TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <EmptyState
    v-else
    title="No releases yet"
    description="Publish a baseline release with the CLI to prepare this application for patches."
  />
</template>
