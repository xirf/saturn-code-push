<script setup lang="ts">
import type { Component } from 'vue'
import PageHeader from '~/components/shared/PageHeader.vue'

export interface SettingsSection {
  label: string
  links: { to: string; label: string; icon: Component }[]
}

defineProps<{ title: string; sections: SettingsSection[] }>()
const route = useRoute()
const description = computed(() =>
  typeof route.meta.description === 'string' ? route.meta.description : undefined,
)
</script>

<template>
  <div class="grid min-w-0 gap-6 lg:grid-cols-[12.5rem_minmax(0,1fr)] lg:gap-12">
    <aside class="hidden lg:block">
      <nav
        :aria-label="`${title} navigation`"
        class="sticky top-22 space-y-5"
      >
        <h2 class="px-3 text-sm font-semibold">{{ title }}</h2>
        <section
          v-for="section in sections"
          :key="section.label"
          class="space-y-0.5"
        >
          <h3 class="px-3 pb-1 text-xs font-medium tracking-wide text-muted-foreground uppercase">
            {{ section.label }}
          </h3>
          <NuxtLink
            v-for="link in section.links"
            :key="link.to"
            :to="link.to"
            :aria-current="route.path === link.to ? 'page' : undefined"
            :class="[
              'flex min-h-9 items-center gap-2.5 rounded-md px-3 text-sm transition-colors',
              route.path === link.to
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
    </aside>
    <div class="min-w-0 space-y-6">
      <div class="lg:hidden">
        <label
          for="settings-page"
          class="sr-only"
        >
          {{ title }} section
        </label>
        <Select
          :model-value="route.path"
          @update:model-value="(path) => navigateTo(String(path))"
        >
          <SelectTrigger
            id="settings-page"
            class="w-full"
            :aria-label="`${title} section`"
          >
            <SelectValue placeholder="Choose a section" />
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
      <PageHeader
        :title="String(route.meta.title ?? title)"
        :description="description"
      />
      <slot />
    </div>
  </div>
</template>
