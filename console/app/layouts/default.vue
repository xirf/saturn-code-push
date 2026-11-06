<script setup lang="ts">
import { BookOpen, Menu, UserRound } from '@lucide/vue'
import { ScrollArea } from '~/components/ui/scroll-area'
import SaturnMark from '~/components/brand/SaturnMark.vue'
import ConsoleNavigation from '~/components/shell/ConsoleNavigation.vue'
import EmptyState from '~/components/shared/EmptyState.vue'
import ErrorNotice from '~/components/shared/ErrorNotice.vue'
import AppSwitcher from '~/features/workspace/components/AppSwitcher.vue'
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
const inApplication = computed(() => typeof route.params.appId === 'string')
const showSwitcher = computed(
  () => inApplication.value || route.path === '/' || route.path === '/new',
)
const missingApplication = computed(
  () => inApplication.value && Boolean(catalog.value) && !workspace.application.value,
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
      <header
        class="sticky top-0 z-40 flex h-14 items-center justify-between gap-3 border-b bg-background/95 px-3 backdrop-blur sm:px-5"
      >
        <div class="flex min-w-0 items-center gap-1 sm:gap-2">
          <Button
            variant="ghost"
            size="icon"
            class="shrink-0 lg:hidden"
            aria-label="Open navigation"
            @click="preferences.mobileNavigationOpen = true"
          >
            <Menu class="size-5" />
          </Button>
          <NuxtLink
            to="/"
            class="flex shrink-0 items-center gap-2 rounded-md px-1.5 py-1 hover:bg-muted"
            aria-label="All applications"
          >
            <SaturnMark class="size-6 text-foreground" />
            <span class="hidden font-heading text-sm font-semibold tracking-tight sm:block">
              saturn
            </span>
          </NuxtLink>
          <nav
            aria-label="Breadcrumb"
            class="flex min-w-0 items-center gap-1 sm:gap-2"
          >
            <span
              class="text-lg font-light text-muted-foreground/40 select-none"
              aria-hidden="true"
            >
              /
            </span>
            <AppSwitcher v-if="showSwitcher" />
            <span
              v-else
              class="truncate text-sm font-medium"
              aria-current="page"
            >
              {{ route.meta.crumb ?? route.meta.title ?? 'Console' }}
            </span>
          </nav>
        </div>
        <div class="flex shrink-0 items-center gap-1.5">
          <Button
            variant="ghost"
            size="sm"
            as-child
            class="hidden sm:inline-flex"
          >
            <NuxtLink to="/docs">
              <BookOpen class="size-3.5" />
              Docs
            </NuxtLink>
          </Button>
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
              <DropdownMenuItem as-child>
                <NuxtLink to="/settings">Server settings</NuxtLink>
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
      <aside
        class="fixed top-14 bottom-0 left-0 z-30 hidden w-56 border-r border-sidebar-border lg:block"
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
      <main
        id="main-content"
        class="min-w-0 lg:pl-56"
      >
        <div class="mx-auto w-full max-w-6xl space-y-6 px-5 py-8 sm:px-8">
          <ErrorNotice
            v-if="logoutError"
            :message="logoutError"
          />
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
            <Skeleton class="h-10 w-64" />
            <Skeleton class="h-28 w-full" />
            <Skeleton class="h-80 w-full" />
          </div>
          <Card v-else-if="missingApplication">
            <EmptyState
              title="Application not found"
              description="This application does not exist on this server, or the link is out of date."
            >
              <Button
                variant="outline"
                as-child
              >
                <NuxtLink to="/">View all applications</NuxtLink>
              </Button>
            </EmptyState>
          </Card>
          <slot v-else-if="catalog" />
        </div>
      </main>
    </div>
  </ScrollArea>
</template>
