<script setup lang="ts">
import { Copy } from '@lucide/vue'
import { toast } from 'vue-sonner'
definePageMeta({
  title: 'General',
  description: 'Connect your tools to this Saturn server.',
  settingsGroup: 'Configuration',
})
const origin = ref('')
onMounted(() => {
  origin.value = window.location.origin
})
async function copy() {
  try {
    await navigator.clipboard.writeText(origin.value)
    toast.success('API origin copied')
  } catch {
    toast.error('Clipboard unavailable. Select and copy the origin manually.')
  }
}
</script>
<template>
  <div class="w-full space-y-5">
    <Card>
      <CardHeader>
        <CardTitle>API origin</CardTitle>
        <CardDescription>Your console and update API share this address.</CardDescription>
      </CardHeader>
      <CardContent class="flex min-w-0 items-center gap-3">
        <code class="min-w-0 flex-1 break-all text-sm">{{ origin }}</code>
        <Button
          variant="outline"
          size="icon"
          aria-label="Copy API origin"
          @click="copy"
        >
          <Copy class="size-4" />
        </Button>
      </CardContent>
    </Card>
    <Card>
      <CardHeader>
        <CardTitle>Connect your CLI</CardTitle>
        <CardDescription>
          Set the server address, then authenticate with an application API key.
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <CodeBlock
          :title="'Shell environment'"
          :code="`SHOREBIRD_HOSTED_URL=${origin}`"
        />
        <p class="text-sm text-muted-foreground">
          For releases and patches, use a key with Publish permission. A deployment-reporting key
          cannot publish updates.
        </p>
        <NuxtLink
          to="/docs/usage"
          class="inline-flex min-h-10 items-center text-sm underline"
        >
          CLI setup guide
        </NuxtLink>
      </CardContent>
    </Card>
  </div>
</template>
