<script setup lang="ts">
import { toast } from 'vue-sonner'
import type { DocumentationPage } from '../types/documentation'

defineProps<{ page: DocumentationPage }>()

async function interact(event: MouseEvent) {
  if (!(event.target instanceof Element)) return
  const button = event.target.closest<HTMLButtonElement>('[data-copy-code]')
  if (button) {
    const code = button.parentElement?.querySelector('code')?.textContent
    if (!code) return
    try {
      await navigator.clipboard.writeText(code)
      toast.success('Code copied')
    } catch {
      toast.error('Clipboard unavailable. Select and copy the code manually.')
    }
    return
  }
  const link = event.target.closest<HTMLAnchorElement>('a[href]')
  if (
    !link ||
    event.ctrlKey ||
    event.metaKey ||
    event.shiftKey ||
    event.altKey ||
    event.button !== 0
  )
    return
  if (link.pathname.startsWith('/docs/') && link.origin === location.origin) {
    event.preventDefault()
    await navigateTo(`${link.pathname}${link.hash}`)
  }
}
</script>

<template>
  <div class="grid min-w-0 gap-12 xl:grid-cols-[minmax(0,1fr)_13rem]">
    <div class="min-w-0">
      <p class="mb-7 text-xs text-muted-foreground">{{ page.section }}</p>
      <!-- Only build-generated Markdown HTML is accepted. Raw source HTML is disabled. -->
      <!-- eslint-disable vue/no-v-html -->
      <article
        class="prose prose-sm prose-neutral max-w-none min-w-0 dark:prose-invert prose-headings:scroll-mt-24 prose-headings:font-heading prose-h1:text-3xl prose-h1:tracking-tight prose-a:break-words prose-a:font-normal prose-a:underline-offset-4 prose-pre:mt-0 prose-pre:mb-0 prose-pre:rounded-lg prose-pre:bg-transparent prose-pre:pt-12 prose-code:before:content-none prose-code:after:content-none"
        :class="page.slug === 'license' && '[&_pre]:whitespace-pre-wrap [&_code]:break-words'"
        aria-label="Documentation content"
        @click="interact"
        v-html="page.html"
      />
      <!-- eslint-enable vue/no-v-html -->
      <p class="mt-10 border-t pt-5 text-xs text-muted-foreground">
        Source:
        <code>{{ page.source }}</code>
      </p>
    </div>
    <aside
      v-if="page.headings.some((heading) => heading.level === 2)"
      class="hidden xl:block"
    >
      <nav
        class="sticky top-24 space-y-3 border-l pl-5"
        aria-label="On this page"
      >
        <p class="text-xs font-medium">On this page</p>
        <a
          v-for="heading in page.headings.filter((heading) => heading.level === 2)"
          :key="heading.id"
          :href="`#${heading.id}`"
          class="block text-xs leading-relaxed text-muted-foreground hover:text-foreground"
        >
          {{ heading.text }}
        </a>
      </nav>
    </aside>
  </div>
</template>
