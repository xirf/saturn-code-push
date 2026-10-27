<script setup lang="ts">
import { Search, ShieldCheck } from '@lucide/vue'
import { InputGroup, InputGroupAddon, InputGroupInput } from '~/components/ui/input-group'
import ReleaseTable from '~/features/distribution/components/ReleaseTable.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'

definePageMeta({
  middleware: 'auth',
  title: 'Releases',
  description: 'Release baselines and platform status.',
})
const workspace = useWorkspace()
const search = ref('')
const releases = computed(() =>
  workspace.releases.value.filter((release) =>
    `${release.version} ${release.flutter_revision}`
      .toLowerCase()
      .includes(search.value.toLowerCase()),
  ),
)
</script>

<template>
  <Card>
    <CardHeader>
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <CardTitle>
            All releases
            <span class="ml-2 text-muted-foreground">{{ workspace.releases.value.length }}</span>
          </CardTitle>
          <CardDescription class="mt-1">
            Published by your CLI and scoped to the selected application.
          </CardDescription>
        </div>
        <InputGroup class="w-full max-w-sm">
          <InputGroupInput
            v-model="search"
            class="pl-9"
            aria-label="Search releases"
            placeholder="Search releases…"
          />
          <InputGroupAddon><Search aria-hidden="true" /></InputGroupAddon>
        </InputGroup>
      </div>
    </CardHeader>
    <CardContent class="px-0"><ReleaseTable :releases="releases" /></CardContent>
    <CardFooter class="gap-2 text-xs text-muted-foreground">
      <ShieldCheck class="size-3.5" />
      Release artifacts are immutable baselines.
    </CardFooter>
  </Card>
</template>
