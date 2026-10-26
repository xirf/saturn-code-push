<script setup lang="ts">
import { ArrowUpRight, BookOpen, Menu } from '@lucide/vue'
import SaturnMark from '~/components/brand/SaturnMark.vue'
import DocumentationNavigation from '~/features/documentation/components/DocumentationNavigation.vue'

const navigationOpen = ref(false)
</script>

<template>
  <div class="min-h-svh bg-background text-foreground">
    <a
      href="#documentation-content"
      class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:rounded-md focus:bg-primary focus:p-3 focus:text-primary-foreground"
    >
      Skip to documentation
    </a>
    <header
      class="sticky top-0 z-30 flex h-16 items-center justify-between gap-4 border-b bg-background/95 px-5 backdrop-blur-sm sm:px-8"
    >
      <div class="flex items-center gap-4">
        <Button
          class="lg:hidden"
          variant="ghost"
          size="icon-sm"
          aria-label="Open documentation navigation"
          @click="navigationOpen = true"
        >
          <Menu class="size-4" />
        </Button>
        <NuxtLink
          to="/docs"
          class="flex items-center gap-2.5 font-heading text-lg font-semibold"
        >
          <SaturnMark class="size-7" />
          saturn
        </NuxtLink>
        <span class="hidden border-l pl-4 text-xs text-muted-foreground sm:block">
          Documentation
        </span>
      </div>
      <Button
        as-child
        variant="outline"
        size="sm"
      >
        <NuxtLink to="/">
          Open console
          <ArrowUpRight class="size-3.5" />
        </NuxtLink>
      </Button>
    </header>
    <aside class="fixed top-16 bottom-0 left-0 hidden w-64 border-r bg-sidebar px-4 py-7 lg:block">
      <DocumentationNavigation />
    </aside>
    <Sheet v-model:open="navigationOpen">
      <SheetContent
        side="left"
        class="w-80"
      >
        <SheetHeader class="mb-4">
          <SheetTitle class="flex items-center gap-2">
            <BookOpen class="size-4" />
            Documentation
          </SheetTitle>
          <SheetDescription>Guides for deploying and using Saturn.</SheetDescription>
        </SheetHeader>
        <div class="min-h-0 flex-1 px-4 pb-5">
          <DocumentationNavigation @navigate="navigationOpen = false" />
        </div>
      </SheetContent>
    </Sheet>
    <main
      id="documentation-content"
      class="min-w-0 px-5 py-9 sm:px-8 lg:ml-64 lg:px-12 lg:py-12"
    >
      <slot />
    </main>
  </div>
</template>
