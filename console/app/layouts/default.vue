<script setup lang="ts">
import { ChevronRight, Menu, LogOut } from '@lucide/vue'
import ConsoleNavigation from '~/components/shell/ConsoleNavigation.vue'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import CreateApplicationDialog from '~/features/applications/components/CreateApplicationDialog.vue'
import { useWorkspace } from '~/features/workspace/composables/useWorkspace'
import { useIdentity } from '~/features/identity/composables/useIdentity'
import { usePreferencesStore } from '~/stores/preferences'
import { errorMessage } from '~/lib/api/client'

const route = useRoute()
const preferences = usePreferencesStore()
const workspace = useWorkspace()
const identity = useIdentity()
const user = identity.session.data
const catalog = workspace.query.data
const loading = computed(() => workspace.query.asyncStatus.value === 'loading')
const queryError = workspace.query.error
const logoutError = ref('')
const title = computed(() => String(route.meta.title ?? 'Overview'))
const showApplicationPicker = computed(() =>
  ['/', '/releases', '/patches', '/analytics'].includes(route.path),
)
const showCreateApplication = computed(
  () => route.path === '/applications' || !workspace.applications.value.length,
)

async function signOut() {
  logoutError.value = ''
  try {
    await identity.logout.mutateAsync()
  } catch (error) {
    logoutError.value = errorMessage(error)
  }
}
</script>

<template>
  <div
    v-if="user"
    class="min-h-svh bg-background text-foreground"
  >
    <a
      href="#main-content"
      class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:rounded-md focus:bg-primary focus:p-3 focus:text-primary-foreground"
    >
      Skip to content
    </a>
    <aside class="fixed inset-y-0 left-0 z-30 hidden w-56 border-r border-sidebar-border lg:block">
      <ConsoleNavigation />
    </aside>
    <Sheet
      :open="preferences.mobileNavigationOpen"
      @update:open="(value) => (preferences.mobileNavigationOpen = value)"
    >
      <SheetContent
        side="left"
        class="w-72 gap-0 p-0"
      >
        <SheetHeader class="sr-only">
          <SheetTitle>Workspace navigation</SheetTitle>
          <SheetDescription>Navigate your Saturn workspace.</SheetDescription>
        </SheetHeader>
        <ConsoleNavigation />
      </SheetContent>
    </Sheet>
    <div class="min-w-0 lg:pl-56">
      <header class="flex min-h-14 items-center justify-between gap-3 border-b px-5 sm:px-8">
        <div class="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            class="lg:hidden"
            aria-label="Open navigation"
            @click="preferences.mobileNavigationOpen = true"
          >
            <Menu class="size-5" />
          </Button>
          <div class="flex items-center gap-3 text-[13px]">
            <span class="text-muted-foreground">Console</span>
            <ChevronRight class="size-3 text-muted-foreground/50" />
            <span class="text-foreground">{{ title }}</span>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class="mr-2 hidden text-xs text-muted-foreground sm:block">
            {{ user.username }}
          </span>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Sign out"
            :disabled="identity.logout.asyncStatus.value === 'loading'"
            @click="signOut"
          >
            <LogOut class="size-4" />
          </Button>
        </div>
      </header>
      <main
        id="main-content"
        class="min-w-0 space-y-6 px-5 py-7 sm:px-8"
      >
        <ErrorNotice
          v-if="logoutError"
          :message="logoutError"
        />
        <div class="flex flex-wrap items-end justify-between gap-5">
          <div>
            <h1 class="font-heading text-2xl font-semibold tracking-[-0.035em]">{{ title }}</h1>
            <p class="mt-1.5 text-[13px] text-muted-foreground">{{ route.meta.description }}</p>
          </div>
          <div class="flex flex-wrap items-center gap-3">
            <Select
              v-if="showApplicationPicker && workspace.applications.value.length"
              :model-value="workspace.appId.value"
              @update:model-value="(value) => (preferences.selectedAppId = String(value))"
            >
              <SelectTrigger
                class="w-56"
                aria-label="Selected application"
              >
                <SelectValue placeholder="Select application" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="app in workspace.applications.value"
                  :key="app.id"
                  :value="app.id"
                >
                  {{ app.display_name }}
                </SelectItem>
              </SelectContent>
            </Select>
            <CreateApplicationDialog v-if="showCreateApplication" />
          </div>
        </div>
        <ErrorNotice
          v-if="queryError"
          :message="errorMessage(queryError)"
        >
          <Button
            class="mt-3"
            variant="outline"
            size="sm"
            @click="workspace.query.refetch()"
          >
            Try again
          </Button>
        </ErrorNotice>
        <div
          v-if="!catalog && loading"
          class="space-y-5"
          role="status"
          aria-label="Loading workspace"
        >
          <Skeleton class="h-28 w-full" />
          <Skeleton class="h-80 w-full" />
        </div>
        <slot v-else-if="catalog" />
      </main>
    </div>
  </div>
</template>
