<script setup lang="ts">
import { GitBranch, ShieldCheck, Activity, BookOpen } from '@lucide/vue'
import SaturnMark from '~/components/brand/SaturnMark.vue'
import SignInForm from '~/features/identity/components/SignInForm.vue'
import LoginOrbit from '~/features/identity/components/LoginOrbit.vue'
import { useSessionQuery } from '~/features/identity/queries/session'

definePageMeta({ layout: false })
useHead({ title: 'Sign in - Saturn' })
const session = useSessionQuery()
watch(
  session.data,
  (user) => {
    if (user) void navigateTo('/')
  },
  { immediate: true },
)
</script>

<template>
  <div class="grid min-h-svh lg:grid-cols-[53%_47%]">
    <section
      class="relative hidden min-h-svh flex-col overflow-hidden border-r border-[#3c3c3c] bg-[#1f1f1f] px-[clamp(2rem,4vw,4rem)] py-10 lg:flex"
    >
      <div
        class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_50%_48%,#3e3e3e77,transparent_65%)]"
      />
      <NuxtLink
        to="/"
        class="relative flex w-fit items-center gap-3 text-xl font-semibold"
      >
        <SaturnMark class="size-9" />
        saturn
        <span
          class="ml-1 border-l border-[#4b4b4b] pl-4 text-[9px] font-semibold tracking-[0.2em] text-[#a5a5a5]"
        >
          CODE PUSH
        </span>
      </NuxtLink>
      <div
        class="relative mx-auto flex w-full max-w-140 flex-1 flex-col items-start justify-center py-12"
      >
        <Badge
          variant="outline"
          class="gap-2 rounded-full border-[#4e4e4e] bg-[#353535] px-3 py-1.5 text-[11px] font-normal text-[#cdcdcd]"
        >
          <span class="size-1 rounded-full bg-current" />
          Your infrastructure. Your updates.
        </Badge>
        <h2
          class="mt-7 font-heading text-[clamp(2.75rem,4.4vw,4.25rem)] leading-[1.12] font-semibold tracking-[-0.045em]"
        >
          Your releases.
          <br />
          <span class="text-[#d9d9d9]">In orbit.</span>
        </h2>
        <p class="mt-5 max-w-sm text-sm leading-[1.85] text-[#a6a6a6]">
          A quieter way to ship. Deliver signed patches, keep releases under control, and see what
          lands.
        </p>
        <LoginOrbit />
        <div class="mx-auto mt-3 flex flex-wrap justify-center gap-5 text-xs text-[#b8b8b8]">
          <span class="flex items-center gap-2">
            <ShieldCheck class="size-4" />
            Signed patches
          </span>
          <span class="flex items-center gap-2">
            <GitBranch class="size-4" />
            Channel delivery
          </span>
          <span class="flex items-center gap-2">
            <Activity class="size-4" />
            Device insights
          </span>
        </div>
      </div>
    </section>
    <main class="flex min-h-svh flex-col items-center justify-center gap-8 px-6 py-16">
      <SignInForm />
      <Button
        as-child
        variant="link"
        size="sm"
        class="text-muted-foreground"
      >
        <NuxtLink to="/docs">
          <BookOpen class="size-3.5" />
          Documentation
        </NuxtLink>
      </Button>
    </main>
  </div>
</template>
