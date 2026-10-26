<script setup lang="ts">
import DocumentationArticle from '~/features/documentation/components/DocumentationArticle.vue'
import { documentation } from '~/features/documentation/content/catalog'

definePageMeta({
  layout: 'documentation',
  validate: (route) => documentation.some((page) => page.slug === route.params.slug),
})
const route = useRoute()
const page = computed(() => documentation.find((page) => page.slug === route.params.slug))
if (!page.value)
  throw createError({ statusCode: 404, statusMessage: 'Documentation page not found' })
useHead(() => ({ title: `${page.value?.title ?? 'Documentation'} - Saturn` }))
</script>

<template>
  <DocumentationArticle
    v-if="page"
    :page="page"
  />
</template>
