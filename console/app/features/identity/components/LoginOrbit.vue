<script setup lang="ts">
import { CircleCheck, ShieldCheck } from '@lucide/vue'
</script>

<template>
  <div
    class="relative my-1 grid h-72 w-full place-items-center"
    aria-hidden="true"
  >
    <svg
      class="absolute inset-0 size-full"
      viewBox="0 0 560 300"
      fill="none"
    >
      <defs>
        <radialGradient
          id="login-planet"
          cx="0.35"
          cy="0.25"
          r="0.8"
        >
          <stop stop-color="#606060" />
          <stop
            offset="1"
            stop-color="#2c2c2c"
          />
        </radialGradient>
        <radialGradient id="login-glow">
          <stop
            stop-color="#b2b2b2"
            stop-opacity="0.07"
          />
          <stop
            offset="1"
            stop-color="#b2b2b2"
            stop-opacity="0"
          />
        </radialGradient>
        <pattern
          id="login-grid"
          width="32"
          height="32"
          patternUnits="userSpaceOnUse"
        >
          <path
            d="M32 0H0V32"
            stroke="#959595"
            stroke-opacity="0.06"
          />
        </pattern>
        <radialGradient id="login-grid-mask">
          <stop stop-color="white" />
          <stop
            offset="1"
            stop-color="black"
          />
        </radialGradient>
        <mask id="login-grid-fade">
          <rect
            width="560"
            height="300"
            fill="url(#login-grid-mask)"
          />
        </mask>
      </defs>
      <rect
        width="560"
        height="300"
        fill="url(#login-grid)"
        mask="url(#login-grid-fade)"
      />
      <ellipse
        cx="280"
        cy="150"
        rx="210"
        ry="125"
        fill="url(#login-glow)"
      />
      <ellipse
        cx="280"
        cy="150"
        rx="195"
        ry="65"
        transform="rotate(-25 280 150)"
        stroke="#737373"
        stroke-opacity="0.55"
      />
      <ellipse
        cx="280"
        cy="150"
        rx="227"
        ry="92"
        transform="rotate(-25 280 150)"
        stroke="#737373"
        stroke-opacity="0.3"
        stroke-dasharray="3 6"
      />
      <circle
        cx="280"
        cy="150"
        r="79"
        fill="url(#login-planet)"
        stroke="#646464"
      />
      <circle
        cx="280"
        cy="150"
        r="26"
        fill="#d6d6d6"
      />
      <ellipse
        cx="280"
        cy="150"
        rx="57"
        ry="18"
        transform="rotate(-30 280 150)"
        stroke="#d6d6d6"
        stroke-width="7"
      />
      <circle
        cx="90"
        cy="192"
        r="3.5"
        fill="#dedede"
        stroke="#373737"
        stroke-width="2"
      />
      <circle
        cx="456"
        cy="88"
        r="2.5"
        fill="#b5b5b5"
      />
    </svg>
    <div
      class="absolute bottom-9 left-1/2 flex -translate-x-1/2 items-center gap-2 whitespace-nowrap rounded-lg border border-[#505050] bg-[#262626]/95 px-3.5 py-2.5 text-xs text-[#d4d4d4] shadow-lg backdrop-blur-md"
    >
      <ShieldCheck class="size-3.5" />
      Signed. Delivered. Verified.
      <CircleCheck class="ml-1 size-3.5" />
    </div>
  </div>
</template>
