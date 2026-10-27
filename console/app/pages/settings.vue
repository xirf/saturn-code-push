<script setup lang="ts">
import { ScrollArea } from '~/components/ui/scroll-area'
import { Activity, GitBranch, HardDrive, KeyRound, Server, UsersRound } from '@lucide/vue'
import { useIdentity } from '~/features/identity/composables/useIdentity'

definePageMeta({ middleware: 'auth' })
const route = useRoute()
const identity = useIdentity()
const isAdmin = computed(() => identity.session.data.value?.role === 'admin')
const sections = computed(() => [
  {
    label: 'Configuration',
    links: [
      { to: '/settings/server/connection', label: 'General', icon: Server },
      ...(isAdmin.value
        ? [{ to: '/settings/server/storage', label: 'Artifact storage', icon: HardDrive }]
        : []),
    ],
  },
  {
    label: 'Connections',
    links: isAdmin.value
      ? [{ to: '/settings/integrations/github', label: 'GitHub', icon: GitBranch }]
      : [],
  },
  ...(isAdmin.value
    ? [
        {
          label: 'Management',
          links: [
            { to: '/settings/management/accounts', label: 'Accounts', icon: UsersRound },
            { to: '/settings/management/api-keys', label: 'API keys', icon: KeyRound },
          ],
        },
        {
          label: 'Compliance',
          links: [{ to: '/settings/management/audit', label: 'Audit history', icon: Activity }],
        },
      ]
    : []),
])
const currentPath = computed(() => route.path)
</script>

<template>
  <div
    class="grid min-w-0 gap-6 lg:-mx-8 lg:-my-7 lg:min-h-[calc(100svh-3.5rem)] lg:grid-cols-[14rem_minmax(0,1fr)] lg:gap-0"
  >
    <aside class="hidden border-r bg-card lg:sticky lg:top-14 lg:block lg:h-[calc(100svh-3.5rem)]">
      <ScrollArea class="h-full">
        <div class="border-b px-5 py-5">
          <h2 class="text-base font-semibold">Settings</h2>
          <p class="mt-1 text-xs leading-relaxed text-muted-foreground">
            Accounts, integrations, and server configuration.
          </p>
        </div>
        <nav
          aria-label="Settings navigation"
          class="space-y-4 p-3 pb-8"
        >
          <section
            v-for="section in sections"
            :key="section.label"
            class="space-y-1"
          >
            <h3
              class="px-3 pt-2 text-sm font-medium uppercase tracking-wider text-muted-foreground"
            >
              {{ section.label }}
            </h3>
            <NuxtLink
              v-for="link in section.links"
              :key="link.to"
              :to="link.to"
              :aria-current="currentPath === link.to ? 'page' : undefined"
              :class="[
                'flex min-h-9 items-center gap-2.5 rounded-md px-3 text-sm transition-colors',
                currentPath === link.to
                  ? 'bg-muted font-medium text-foreground'
                  : 'text-muted-foreground hover:bg-muted/70 hover:text-foreground',
              ]"
            >
              <component
                :is="link.icon"
                class="size-4 shrink-0"
                aria-hidden="true"
              />
              {{ link.label }}
            </NuxtLink>
          </section>
        </nav>
      </ScrollArea>
    </aside>

    <div class="flex w-full min-w-0 justify-center">
      <div class="w-full min-w-0 max-w-5xl px-5 py-6 sm:px-7 lg:px-8 lg:py-7">
        <div class="mb-5 lg:hidden">
          <h2 class="text-lg font-semibold">Settings</h2>
          <p class="mt-1 text-sm text-muted-foreground">
            Accounts, integrations, and server configuration.
          </p>
        </div>
        <div class="mb-5 lg:hidden">
          <label
            for="settings-page"
            class="sr-only"
          >
            Settings section
          </label>
          <Select
            :model-value="currentPath"
            @update:model-value="(path) => navigateTo(String(path))"
          >
            <SelectTrigger
              id="settings-page"
              class="w-full"
              aria-label="Settings section"
            >
              <SelectValue placeholder="Choose a settings section" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup
                v-for="section in sections"
                :key="section.label"
              >
                <SelectLabel>{{ section.label }}</SelectLabel>
                <SelectItem
                  v-for="link in section.links"
                  :key="link.to"
                  :value="link.to"
                >
                  {{ link.label }}
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
        <header class="mb-5">
          <h1 class="font-heading text-2xl font-semibold tracking-[-0.035em]">
            {{ route.meta.title }}
          </h1>
          <p class="mt-1.5 text-[13px] text-muted-foreground">{{ route.meta.description }}</p>
        </header>
        <NuxtPage />
      </div>
    </div>
  </div>
</template>
