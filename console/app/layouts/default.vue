<script setup lang="ts">
import { ChevronRight, Menu, UserRound } from '@lucide/vue'
import { ScrollArea } from '~/components/ui/scroll-area'
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
  () =>
    identity.session.data.value?.role === 'admin' &&
    (route.path === '/applications' || !workspace.applications.value.length),
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
  <ScrollArea
    v-if="user"
    class="h-svh w-full"
  >
    <div class="min-h-svh bg-background text-foreground">
      <a
        href="#main-content"
        class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:rounded-md focus:bg-primary focus:p-3 focus:text-primary-foreground"
      >
        Skip to content
      </a>
      <aside
        class="fixed inset-y-0 left-0 z-30 hidden w-56 border-r border-sidebar-border lg:block"
      >
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
        <header
          class="sticky top-0 z-40 flex min-h-14 items-center justify-between gap-3 border-b bg-background/95 px-5 backdrop-blur sm:px-8"
        >
          <div class="flex min-w-0 items-center gap-2 sm:gap-3">
            <Button
              variant="ghost"
              size="icon"
              class="shrink-0 lg:hidden"
              aria-label="Open navigation"
              @click="preferences.mobileNavigationOpen = true"
            >
              <Menu class="size-5" />
            </Button>
            <nav
              aria-label="Breadcrumb"
              class="flex min-w-0 items-center gap-2 text-sm sm:gap-3"
            >
              <NuxtLink
                to="/"
                class="shrink-0 text-muted-foreground hover:text-foreground"
              >
                Console
              </NuxtLink>
              <ChevronRight class="size-3 shrink-0 text-muted-foreground/50" />
              <template v-if="route.path.startsWith('/settings/')">
                <NuxtLink
                  to="/settings"
                  class="shrink-0 text-muted-foreground hover:text-foreground"
                >
                  Settings
                </NuxtLink>
                <ChevronRight class="size-4 shrink-0 text-muted-foreground/50" />
                <template v-if="route.meta.settingsGroup">
                  <span class="text-muted-foreground">
                    {{ route.meta.settingsGroup }}
                  </span>
                  <ChevronRight class="size-4 text-muted-foreground/50" />
                </template>
              </template>
              <span
                class="min-w-0 truncate text-foreground"
                aria-current="page"
              >
                {{ title }}
              </span>
            </nav>
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <span class="mr-2 hidden text-xs text-muted-foreground sm:block">
              {{ user.username }}
            </span>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Open account menu"
                >
                  <UserRound class="size-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align="end"
                class="w-56"
              >
                <DropdownMenuLabel>
                  <div class="font-medium">{{ user.username }}</div>
                  <div class="text-xs font-normal text-muted-foreground">{{ user.role }}</div>
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuItem as-child>
                  <NuxtLink to="/account">Account &amp; security</NuxtLink>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  :disabled="identity.logout.asyncStatus.value === 'loading'"
                  @select="signOut"
                >
                  Sign out
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
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
          <div
            v-if="!route.path.startsWith('/settings')"
            :class="[
              'flex flex-wrap items-end justify-between gap-5',
              route.path === '/account' && 'mx-auto w-full max-w-5xl',
            ]"
          >
            <div>
              <h1 class="font-heading text-2xl font-semibold tracking-[-0.035em]">{{ title }}</h1>
              <p class="mt-1.5 text-sm text-muted-foreground">{{ route.meta.description }}</p>
            </div>
            <div class="flex flex-wrap items-center gap-3">
              <Select
                v-if="route.path === '/analytics'"
                :model-value="String(preferences.periodDays)"
                @update:model-value="(value) => (preferences.periodDays = Number(value))"
              >
                <SelectTrigger
                  class="w-40 text-sm"
                  aria-label="Reporting period"
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="7">Last 7 UTC days</SelectItem>
                  <SelectItem value="14">Last 14 UTC days</SelectItem>
                  <SelectItem value="30">Last 30 UTC days</SelectItem>
                </SelectContent>
              </Select>
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
  </ScrollArea>
</template>
