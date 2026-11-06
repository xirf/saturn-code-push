<script setup lang="ts">
const props = withDefaults(
  defineProps<{ name: string; seed?: string; size?: 'sm' | 'md' | 'lg' }>(),
  {
    seed: '',
    size: 'md',
  },
)

// A stable colour per application keeps apps recognisable across the switcher, cards and sidebar.
const hue = computed(() => {
  const value = props.seed || props.name
  let hash = 0
  for (let index = 0; index < value.length; index += 1) {
    hash = (hash * 31 + value.charCodeAt(index)) % 360
  }
  return hash
})
const initial = computed(() => (props.name.trim()[0] ?? '?').toUpperCase())
const sizes = {
  sm: 'size-5 rounded-[5px] text-[10px]',
  md: 'size-8 rounded-lg text-sm',
  lg: 'size-12 rounded-xl text-xl',
}
</script>

<template>
  <span
    aria-hidden="true"
    :class="[
      'inline-flex shrink-0 items-center justify-center font-heading font-semibold text-white shadow-inner ring-1 ring-white/10',
      sizes[size],
    ]"
    :style="{
      backgroundImage: `linear-gradient(135deg, hsl(${hue} 62% 52%), hsl(${(hue + 48) % 360} 62% 34%))`,
    }"
  >
    {{ initial }}
  </span>
</template>
