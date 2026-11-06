<script setup lang="ts">
import { Check, ChevronsUpDown, LayoutGrid, Plus, Search } from '@lucide/vue'
import AppAvatar from '~/components/shared/AppAvatar.vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { useWorkspace } from '../composables/useWorkspace'

const workspace = useWorkspace()
const route = useRoute()
const identity = useIdentity()
const search = ref('')
const open = ref(false)
const sections = ['releases', 'patches', 'analytics', 'settings']
// Keep the visitor on the same kind of page when switching application.
const section = computed(() => {
  const segment = route.path.split('/')[3] ?? ''
  return sections.includes(segment) ? segment : ''
})
const visible = computed(() =>
  workspace.applications.value.filter((app) =>
    app.display_name.toLowerCase().includes(search.value.trim().toLowerCase()),
  ),
)
const searchField = useTemplateRef<{ $el: HTMLInputElement }>('searchField')
watch(open, (value) => {
  if (!value) {
    search.value = ''
    return
  }
  // Reka moves focus to the menu itself on open; hand it to the search box afterwards.
  setTimeout(() => searchField.value?.$el.focus(), 30)
})
function target(id: string) {
  return `/apps/${encodeURIComponent(id)}${section.value ? `/${section.value}` : ''}`
}
</script>

<template>
  <DropdownMenu v-model:open="open">
    <DropdownMenuTrigger as-child>
      <Button
        variant="ghost"
        size="sm"
        class="h-9 max-w-64 min-w-0 shrink gap-2 px-2"
        aria-label="Switch application"
      >
        <template v-if="workspace.application.value">
          <AppAvatar
            :name="workspace.application.value.display_name"
            :seed="workspace.application.value.id"
            size="sm"
          />
          <span class="truncate text-sm font-medium">
            {{ workspace.application.value.display_name }}
          </span>
        </template>
        <span
          v-else
          class="text-sm font-medium"
        >
          All applications
        </span>
        <ChevronsUpDown class="size-3.5 shrink-0 text-muted-foreground" />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent
      align="start"
      class="w-72!"
    >
      <div class="relative px-1 pt-1 pb-1">
        <Search
          class="pointer-events-none absolute top-1/2 left-3.5 size-3.5 -translate-y-1/2 text-muted-foreground"
          aria-hidden="true"
        />
        <Input
          ref="searchField"
          v-model="search"
          class="h-8 pl-8 text-sm"
          placeholder="Find application…"
          aria-label="Find application"
          @keydown.stop
        />
      </div>
      <DropdownMenuLabel class="text-xs font-normal text-muted-foreground">
        Applications
      </DropdownMenuLabel>
      <DropdownMenuItem
        v-for="app in visible"
        :key="app.id"
        as-child
      >
        <NuxtLink
          :to="target(app.id)"
          class="gap-2"
        >
          <AppAvatar
            :name="app.display_name"
            :seed="app.id"
            size="sm"
          />
          <span class="min-w-0 flex-1 truncate">{{ app.display_name }}</span>
          <Check
            v-if="app.id === workspace.appId.value"
            class="size-4"
            aria-label="Current application"
          />
        </NuxtLink>
      </DropdownMenuItem>
      <p
        v-if="!visible.length"
        class="px-2 py-3 text-center text-xs text-muted-foreground"
      >
        No application matches “{{ search }}”.
      </p>
      <DropdownMenuSeparator />
      <DropdownMenuItem as-child>
        <NuxtLink to="/">
          <LayoutGrid class="size-4" />
          All applications
        </NuxtLink>
      </DropdownMenuItem>
      <DropdownMenuItem
        v-if="identity.session.data.value?.role === 'admin'"
        as-child
      >
        <NuxtLink to="/new">
          <Plus class="size-4" />
          Create application
        </NuxtLink>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
