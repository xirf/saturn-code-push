<script setup lang="ts">
definePageMeta({ middleware: 'auth' })
await navigateTo('/settings/server/connection', { replace: true })
</script>
<template><div /></template>
