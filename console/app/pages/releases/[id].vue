<script setup lang="ts">
import DeploymentHistory from '~/features/deployments/components/DeploymentHistory.vue'
import { ArrowLeft } from '@lucide/vue'
import EmptyState from '~/components/shared/EmptyState.vue'
import PatchTable from '~/features/distribution/components/PatchTable.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { usePreferencesStore } from '~/stores/preferences'
import { formatBytes, formatDate } from '~/lib/format'

definePageMeta({
  middleware: 'auth',
  title: 'Release details',
  description: 'Baseline artifacts and patches for this release.',
})
const route = useRoute()
const workspace = useWorkspace()
const preferences = usePreferencesStore()
const release = computed(() =>
  workspace.query.data.value?.releases.find((item) => String(item.id) === route.params.id),
)
const artifacts = computed(() =>
  workspace.artifacts.value.filter(
    (item) => item.release_id === release.value?.id && item.patch_id === null,
  ),
)
const patches = computed(() =>
  workspace.patches.value.filter((item) => item.release_id === release.value?.id),
)
watch(
  release,
  (value) => {
    if (value) preferences.selectedAppId = value.app_id
  },
  { immediate: true },
)
</script>

<template>
  <div class="space-y-6">
    <Button
      variant="ghost"
      as-child
    >
      <NuxtLink to="/releases">
        <ArrowLeft class="size-4" />
        All releases
      </NuxtLink>
    </Button>
    <template v-if="release">
      <DeploymentHistory
        :key="release.id"
        :release="release"
        :patches="patches"
      />
      <Card>
        <CardHeader>
          <CardTitle>{{ release.version }}</CardTitle>
          <CardDescription>
            Created {{ formatDate(release.created_at) }} · Flutter revision
            {{ release.flutter_revision }}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div class="mb-5 flex flex-wrap gap-2">
            <Badge
              v-for="(platform, name) in release.platforms"
              :key="name"
              variant="outline"
            >
              {{ name }} · {{ platform.status }}
            </Badge>
          </div>
          <Table v-if="artifacts.length">
            <TableHeader>
              <TableRow>
                <TableHead>Platform</TableHead>
                <TableHead>Architecture</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Upload</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="artifact in artifacts"
                :key="artifact.id"
              >
                <TableCell>{{ artifact.platform }}</TableCell>
                <TableCell>{{ artifact.arch }}</TableCell>
                <TableCell>{{ formatBytes(artifact.size) }}</TableCell>
                <TableCell>
                  <Badge variant="secondary">
                    {{ artifact.uploaded ? 'Complete' : 'Pending' }}
                  </Badge>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
          <EmptyState
            v-else
            title="No baseline artifacts"
            description="Upload release artifacts with the CLI before activating this release."
          />
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>Release patches</CardTitle>
        </CardHeader>
        <CardContent class="px-0">
          <PatchTable :patches="patches" />
        </CardContent>
      </Card>
    </template>
    <Card v-else>
      <EmptyState
        title="Release not found"
        description="The requested release is not present in this workspace."
      />
    </Card>
  </div>
</template>
