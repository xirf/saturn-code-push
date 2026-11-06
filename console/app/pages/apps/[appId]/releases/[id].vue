<script setup lang="ts">
import DeploymentHistory from '~/features/deployments/components/DeploymentHistory.vue'
import { ArrowLeft } from '@lucide/vue'
import EmptyState from '~/components/shared/EmptyState.vue'
import PatchTable from '~/features/distribution/components/PatchTable.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { formatBytes, formatDate } from '~/lib/format'

definePageMeta({
  middleware: 'auth',
  title: 'Release details',
  description: 'Baseline artifacts and patches for this release.',
})
const route = useRoute()
const workspace = useWorkspace()
const release = computed(() =>
  workspace.releases.value.find((item) => String(item.id) === route.params.id),
)
const artifacts = computed(() =>
  workspace.artifacts.value.filter(
    (item) => item.release_id === release.value?.id && item.patch_id === null,
  ),
)
const patches = computed(() =>
  workspace.patches.value.filter((item) => item.release_id === release.value?.id),
)
</script>

<template>
  <div class="space-y-6">
    <Button
      variant="ghost"
      size="sm"
      class="-ml-2"
      as-child
    >
      <NuxtLink :to="workspace.path('releases')">
        <ArrowLeft class="size-4" />
        All releases
      </NuxtLink>
    </Button>
    <template v-if="release">
      <header class="space-y-1.5">
        <h1 class="font-heading text-2xl font-semibold tracking-[-0.035em]">
          Release {{ release.version }}
        </h1>
        <p class="text-sm text-muted-foreground">
          Created {{ formatDate(release.created_at) }} · Flutter revision
          <code class="text-xs">{{ release.flutter_revision }}</code>
        </p>
      </header>
      <DeploymentHistory
        :key="release.id"
        :release="release"
        :patches="patches"
      />
      <Card>
        <CardHeader>
          <CardTitle>Baseline artifacts</CardTitle>
          <CardDescription>The immutable builds that patches are compared against.</CardDescription>
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
        description="The requested release is not part of this application."
      />
    </Card>
  </div>
</template>
