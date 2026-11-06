<script setup lang="ts">
import { GitBranch } from '@lucide/vue'
import EmptyState from '~/components/shared/EmptyState.vue'
import { usePatchRows } from '../composables/usePatchRows'
import PatchActions from './PatchActions.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import PatchAssetsDialog from '~/features/artifacts/components/PatchAssetsDialog.vue'
import type { Patch } from '../types/patch'

const props = defineProps<{ patches: Patch[] }>()
const workspace = useWorkspace()
const rows = usePatchRows(toRef(props, 'patches'))
</script>

<template>
  <Table v-if="patches.length">
    <TableHeader>
      <TableRow>
        <TableHead>Patch</TableHead>
        <TableHead>Release</TableHead>
        <TableHead>Channels</TableHead>
        <TableHead>Artifacts</TableHead>
        <TableHead>Assets</TableHead>
        <TableHead>Status</TableHead>
        <TableHead class="text-right">Actions</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow
        v-for="{ patch, releaseVersion, channels, artifacts, status } in rows"
        :key="patch.id"
      >
        <TableCell class="font-medium">
          <span class="flex items-center gap-2">
            <GitBranch class="size-4 text-muted-foreground" />
            Patch #{{ patch.number }}
          </span>
        </TableCell>
        <TableCell>
          <NuxtLink
            :to="workspace.path(`releases/${patch.release_id}`)"
            class="font-mono text-xs hover:underline"
          >
            {{ releaseVersion }}
          </NuxtLink>
        </TableCell>
        <TableCell>
          <div class="flex flex-wrap gap-1">
            <Badge
              v-for="channel in channels"
              :key="channel"
              variant="outline"
            >
              {{ channel }}
            </Badge>
            <span
              v-if="!patch.channel_ids.length"
              class="text-xs text-muted-foreground"
            >
              Unpublished
            </span>
          </div>
        </TableCell>
        <TableCell class="text-xs text-muted-foreground">
          {{ artifacts }}
        </TableCell>
        <TableCell class="text-xs text-muted-foreground">
          <PatchAssetsDialog
            :patch-id="patch.id"
            :patch-number="patch.number"
          />
        </TableCell>
        <TableCell>
          <Badge :variant="patch.is_rolled_back ? 'outline' : 'secondary'">
            {{ status }}
          </Badge>
        </TableCell>
        <TableCell>
          <PatchActions :patch="patch" />
        </TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <EmptyState
    v-else
    title="No patches yet"
    description="Publish a signed patch with the CLI, then promote it to a delivery channel."
  />
</template>
