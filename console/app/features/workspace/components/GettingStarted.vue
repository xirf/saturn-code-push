<script setup lang="ts">
import { Check } from '@lucide/vue'
import CodeBlock from '~/components/ui/code-block/CodeBlock.vue'
import { useWorkspace } from '../composables/useWorkspace'

const workspace = useWorkspace()
const origin = ref('')
onMounted(() => {
  origin.value = window.location.origin
})
const config = computed(
  () => `app_id: ${workspace.appId.value}\nbase_url: ${origin.value}\nauto_update: true`,
)
const release = computed(
  () =>
    `export SHOREBIRD_HOSTED_URL=${origin.value}\nexport SHOREBIRD_TOKEN=<your Publish API key>\n\nshorebird release android`,
)
const patch = computed(() => `shorebird patch android --release-version=1.0.0+1 --track=staging`)
const steps = computed(() => [
  {
    title: 'Point your Flutter app at Saturn',
    done: false,
    code: config.value,
    name: 'shorebird.yaml',
  },
  {
    title: 'Create a Publish API key',
    done: false,
    link: { to: workspace.path('settings/api-keys'), label: 'Create API key' },
    body: 'Keys are scoped to this application. The full key is shown once, so keep it in your CI secret store.',
  },
  {
    title: 'Publish your first release',
    done: workspace.releases.value.length > 0,
    code: release.value,
    name: 'Terminal',
  },
  {
    title: 'Ship a patch',
    done: workspace.patches.value.length > 0,
    code: patch.value,
    name: 'Terminal',
    body: 'Then promote it from the Patches tab once it looks right on staging.',
  },
])
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>Get started</CardTitle>
      <CardDescription>
        Four steps from a new application to a patch on real devices.
      </CardDescription>
    </CardHeader>
    <CardContent>
      <ol class="space-y-8">
        <li
          v-for="(step, index) in steps"
          :key="step.title"
          class="grid min-w-0 grid-cols-[2rem_minmax(0,1fr)] gap-4"
        >
          <span
            :class="[
              'flex size-8 items-center justify-center rounded-full border text-sm font-medium',
              step.done ? 'border-emerald-400/40 bg-emerald-400/10 text-emerald-400' : 'bg-muted',
            ]"
          >
            <Check
              v-if="step.done"
              class="size-4"
              aria-label="Done"
            />
            <template v-else>{{ index + 1 }}</template>
          </span>
          <div class="min-w-0 space-y-3 pt-1">
            <h3 class="text-sm font-medium">{{ step.title }}</h3>
            <p
              v-if="step.body"
              class="text-sm text-muted-foreground"
            >
              {{ step.body }}
            </p>
            <CodeBlock
              v-if="step.code"
              :title="step.name"
              :code="step.code"
            />
            <Button
              v-if="step.link"
              variant="outline"
              size="sm"
              as-child
            >
              <NuxtLink :to="step.link.to">{{ step.link.label }}</NuxtLink>
            </Button>
          </div>
        </li>
      </ol>
    </CardContent>
    <CardFooter class="text-xs text-muted-foreground">
      Need the long version?
      <NuxtLink
        to="/docs/usage"
        class="ml-1 underline underline-offset-4"
      >
        Read the CLI guide
      </NuxtLink>
    </CardFooter>
  </Card>
</template>
