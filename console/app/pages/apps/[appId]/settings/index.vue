<script setup lang="ts">
import { Copy } from '@lucide/vue'
import { toast } from 'vue-sonner'
import CodeBlock from '~/components/ui/code-block/CodeBlock.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { formatDate } from '~/lib/format'

definePageMeta({
  title: 'General',
  description: 'Identifiers and connection details for this application.',
})
const workspace = useWorkspace()
const origin = ref('')
onMounted(() => {
  origin.value = window.location.origin
})
const rows = computed(() => [
  {
    label: 'Application name',
    hint: 'Displayed throughout the console.',
    value: workspace.application.value?.display_name ?? '',
    copy: false,
  },
  {
    label: 'Application ID',
    hint: 'Used as app_id in shorebird.yaml and by your CI.',
    value: workspace.appId.value,
    copy: true,
  },
  {
    label: 'Server URL',
    hint: 'The base_url devices and the CLI talk to.',
    value: origin.value,
    copy: true,
  },
  {
    label: 'Created',
    hint: 'When this application was registered.',
    value: formatDate(workspace.application.value?.created_at ?? ''),
    copy: false,
  },
])
const config = computed(
  () => `app_id: ${workspace.appId.value}\nbase_url: ${origin.value}\nauto_update: true`,
)

async function copy(label: string, value: string) {
  try {
    await navigator.clipboard.writeText(value)
    toast.success(`${label} copied`)
  } catch {
    toast.error('Clipboard unavailable. Select and copy the value manually.')
  }
}
</script>

<template>
  <div class="space-y-6">
    <Card class="gap-0 py-0">
      <dl class="divide-y">
        <div
          v-for="row in rows"
          :key="row.label"
          class="grid items-center gap-2 px-6 py-4 sm:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)] sm:gap-6"
        >
          <div>
            <dt class="text-sm font-medium">{{ row.label }}</dt>
            <p class="text-xs text-muted-foreground">{{ row.hint }}</p>
          </div>
          <dd class="flex min-w-0 items-center gap-2">
            <code class="min-w-0 flex-1 truncate rounded-md border bg-muted/30 px-3 py-2 text-sm">
              {{ row.value }}
            </code>
            <Button
              v-if="row.copy"
              variant="outline"
              size="icon"
              :aria-label="`Copy ${row.label}`"
              @click="copy(row.label, row.value)"
            >
              <Copy class="size-4" />
            </Button>
          </dd>
        </div>
      </dl>
    </Card>
    <Card>
      <CardHeader>
        <CardTitle>Flutter configuration</CardTitle>
        <CardDescription>
          Bundle this file with your app, then publish releases with the Shorebird CLI.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-3">
        <CodeBlock
          title="shorebird.yaml"
          :code="config"
        />
        <NuxtLink
          to="/docs/usage"
          class="inline-flex text-sm underline underline-offset-4"
        >
          CLI setup guide
        </NuxtLink>
      </CardContent>
    </Card>
    <Card>
      <CardHeader>
        <CardTitle>Delivery channels</CardTitle>
        <CardDescription>
          Patches are promoted to a channel; devices follow the channel they subscribe to.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ul
          v-if="workspace.channels.value.length"
          class="flex flex-wrap gap-2"
        >
          <li
            v-for="channel in workspace.channels.value"
            :key="channel.id"
          >
            <Badge variant="outline">{{ channel.name }}</Badge>
          </li>
        </ul>
        <p
          v-else
          class="text-sm text-muted-foreground"
        >
          No delivery channels found for this application.
        </p>
      </CardContent>
    </Card>
  </div>
</template>
