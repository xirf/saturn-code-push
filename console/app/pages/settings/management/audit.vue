<script setup lang="ts">
import { useQuery } from '@pinia/colada'
import { ChevronLeft, ChevronRight, RefreshCw } from '@lucide/vue'
import { auditSchema } from '~/features/access/types/access'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useSessionScope } from '~/features/identity/stores/session-scope'
import { request, errorMessage } from '~/lib/api/client'

definePageMeta({
  middleware: 'auth',
  title: 'Audit history',
  settingsGroup: 'Compliance',
  description: 'Review management changes and the account or API key responsible.',
})

const identity = useIdentity()
const scope = useSessionScope()
const isAdmin = computed(() => identity.session.data.value?.role === 'admin')
const cursors = ref<(number | null)[]>([null])
const before = computed(() => cursors.value.at(-1) ?? null)
const audit = useQuery({
  key: () => ['private', scope.generation, 'audit', before.value],
  enabled: () => isAdmin.value,
  query: async ({ signal }) =>
    auditSchema.parse(
      await request(`/admin/audit${before.value ? `?before=${before.value}` : ''}`, { signal }),
    ),
})
const loading = computed(() => audit.asyncStatus.value === 'loading')
const events = computed(() => audit.data.value?.events ?? [])
const timestamp = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'medium' })
function older() {
  const last = events.value.at(-1)
  if (!loading.value && !audit.error.value && events.value.length === 100 && last) {
    cursors.value.push(last.id)
  }
}
</script>

<template>
  <div class="w-full space-y-5">
    <Card v-if="isAdmin">
      <CardHeader class="flex flex-wrap items-start justify-between gap-4">
        <div class="space-y-1.5">
          <CardTitle>Recent changes</CardTitle>
          <CardDescription>
            Successful changes, newest first. Times are shown in your local timezone.
          </CardDescription>
        </div>
        <Button
          variant="outline"
          size="sm"
          :disabled="loading"
          @click="audit.refresh()"
        >
          <RefreshCw
            class="size-4"
            :class="{ 'animate-spin': loading }"
            aria-hidden="true"
          />
          Refresh
        </Button>
      </CardHeader>
      <CardContent
        class="p-0"
        :aria-busy="loading"
      >
        <p
          v-if="audit.error.value"
          role="alert"
          class="px-6 py-8 text-sm text-destructive"
        >
          {{ errorMessage(audit.error.value) }}
        </p>
        <p
          v-else-if="loading"
          role="status"
          class="px-6 py-8 text-sm text-muted-foreground"
        >
          Loading audit history…
        </p>
        <p
          v-else-if="!events.length"
          class="px-6 py-8 text-sm text-muted-foreground"
        >
          No audit events to show.
        </p>
        <div v-else>
          <div class="space-y-3 p-3 sm:p-4 md:hidden">
            <article
              v-for="event in events"
              :key="event.id"
              class="space-y-3 rounded-lg border bg-background p-4"
            >
              <div class="flex items-start justify-between gap-3">
                <time
                  class="text-sm font-medium tabular-nums"
                  :datetime="event.occurred_at"
                >
                  {{ timestamp.format(new Date(event.occurred_at)) }}
                </time>
                <Badge
                  variant="secondary"
                  class="shrink-0 font-mono text-xs"
                >
                  {{ event.action }}
                </Badge>
              </div>
              <div class="space-y-1">
                <p class="text-sm font-medium">{{ event.actor_name }}</p>
                <p class="text-xs text-muted-foreground">{{ event.actor_kind }}</p>
              </div>
              <code class="block break-all text-xs text-muted-foreground">
                {{ event.resource }}
              </code>
            </article>
          </div>
          <div class="hidden md:block">
            <Table>
              <TableHeader class="bg-muted/50">
                <TableRow>
                  <TableHead>When</TableHead>
                  <TableHead>Actor</TableHead>
                  <TableHead>Method</TableHead>
                  <TableHead>Resource</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-for="event in events"
                  :key="event.id"
                  class="hover:bg-muted/30"
                >
                  <TableCell class="whitespace-nowrap px-4 py-3 align-top text-sm tabular-nums">
                    <time :datetime="event.occurred_at">
                      {{ timestamp.format(new Date(event.occurred_at)) }}
                    </time>
                  </TableCell>
                  <TableCell class="px-4 py-3 align-top">
                    <span class="font-medium">{{ event.actor_name }}</span>
                    <span class="mt-1 block text-xs text-muted-foreground">
                      {{ event.actor_kind }}
                    </span>
                  </TableCell>
                  <TableCell class="px-4 py-3 align-top">
                    <Badge
                      variant="secondary"
                      class="font-mono text-xs"
                    >
                      {{ event.action }}
                    </Badge>
                  </TableCell>
                  <TableCell
                    class="max-w-md break-all px-4 py-3 align-top font-mono text-xs leading-relaxed"
                  >
                    {{ event.resource }}
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
        </div>
        <nav
          aria-label="Audit history pages"
          class="flex flex-wrap items-center justify-between gap-3 border-t px-4 py-4 sm:px-6"
        >
          <p
            role="status"
            aria-live="polite"
            class="text-xs text-muted-foreground"
          >
            Page {{ cursors.length }} ·
            {{
              loading ? 'Loading…' : `${events.length} ${events.length === 1 ? 'event' : 'events'}`
            }}
            · Up to 100 per page
          </p>
          <div class="flex flex-wrap gap-2">
            <Button
              v-if="before"
              variant="ghost"
              size="sm"
              :disabled="loading"
              @click="cursors = [null]"
            >
              Latest
            </Button>
            <Button
              variant="outline"
              size="sm"
              :disabled="loading || cursors.length === 1"
              @click="cursors.pop()"
            >
              <ChevronLeft
                class="size-4"
                aria-hidden="true"
              />
              Newer events
            </Button>
            <Button
              variant="outline"
              size="sm"
              :disabled="loading || Boolean(audit.error.value) || events.length !== 100"
              @click="older"
            >
              Older events
              <ChevronRight
                class="size-4"
                aria-hidden="true"
              />
            </Button>
          </div>
        </nav>
      </CardContent>
    </Card>
    <p
      v-else
      role="alert"
      class="text-sm text-muted-foreground"
    >
      Only administrators can view audit history.
    </p>
  </div>
</template>
