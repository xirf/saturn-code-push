<script setup lang="ts">
import { Search } from '@lucide/vue'
import { documentation } from '../content/catalog'

const emit = defineEmits<{ navigate: [] }>()
const route = useRoute()
const search = ref('')
const pages = computed(() => {
  const query = search.value.trim().toLowerCase()
  return documentation.filter((page) =>
    `${page.title} ${page.description} ${page.searchText}`.toLowerCase().includes(query),
  )
})
const sections = computed(() => [...new Set(pages.value.map((page) => page.section))])
</script>

<template>
  <div class="flex h-full flex-col gap-6">
    <div class="relative">
      <Search class="pointer-events-none absolute top-2.5 left-3 size-4 text-muted-foreground" />
      <Input
        v-model="search"
        class="pl-9"
        placeholder="Search documentation…"
        aria-label="Search documentation"
      />
    </div>
    <nav
      class="min-h-0 space-y-6 overflow-y-auto"
      aria-label="Documentation"
    >
      <div
        v-for="section in sections"
        :key="section"
        class="space-y-1"
      >
        <p class="mb-2 px-3 text-[11px] font-medium text-muted-foreground">{{ section }}</p>
        <NuxtLink
          v-for="page in pages.filter((page) => page.section === section)"
          :key="page.slug"
          :to="`/docs/${page.slug}`"
          :aria-current="route.path === `/docs/${page.slug}` ? 'page' : undefined"
          :class="[
            'block rounded-md px-3 py-2 text-[13px] hover:bg-accent hover:text-accent-foreground',
            route.path === `/docs/${page.slug}`
              ? 'bg-accent font-medium text-foreground'
              : 'text-muted-foreground',
          ]"
          @click="emit('navigate')"
        >
          {{ page.title }}
        </NuxtLink>
      </div>
      <p
        v-if="!pages.length"
        class="px-3 text-sm text-muted-foreground"
      >
        No guides match your search.
      </p>
    </nav>
  </div>
</template>
