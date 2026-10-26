<script setup lang="ts">
import { ArrowUpRight } from '@lucide/vue'
import { documentation } from '~/features/documentation/content/catalog'

definePageMeta({ layout: 'documentation' })
useHead({ title: 'Documentation - Saturn' })
const sections = [...new Set(documentation.map((page) => page.section))]
</script>

<template>
  <div class="space-y-10">
    <div class="max-w-2xl space-y-3">
      <p class="text-xs text-muted-foreground">Saturn Code Push</p>
      <h1 class="font-heading text-3xl font-semibold tracking-tight">Documentation</h1>
      <p class="text-sm leading-relaxed text-muted-foreground">
        Set up your server, ship Dart patches, and publish asset updates. Start with the Shorebird
        guide to understand what stays familiar and which steps Saturn adds.
      </p>
    </div>
    <section
      v-for="section in sections"
      :key="section"
      class="space-y-4"
    >
      <h2 class="text-sm font-medium">{{ section }}</h2>
      <div class="grid gap-3 md:grid-cols-2 2xl:grid-cols-3">
        <NuxtLink
          v-for="page in documentation.filter((page) => page.section === section)"
          :key="page.slug"
          :to="`/docs/${page.slug}`"
          class="group flex items-start gap-4 rounded-xl border bg-card p-5 transition-colors hover:bg-accent/50"
        >
          <div class="flex-1 space-y-2">
            <h3 class="text-sm font-medium">{{ page.title }}</h3>
            <p class="text-xs leading-relaxed text-muted-foreground">{{ page.description }}</p>
          </div>
          <ArrowUpRight class="size-4 shrink-0 text-muted-foreground group-hover:text-foreground" />
        </NuxtLink>
      </div>
    </section>
  </div>
</template>
