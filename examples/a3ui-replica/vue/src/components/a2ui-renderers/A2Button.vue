<template>
  <button
    :class="[variantClass]"
    class="px-4 py-2 rounded-md font-medium text-sm transition-all duration-150 cursor-pointer"
  >
    {{ component.label }}
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const variantClass = computed(() => {
  const v = props.component.variant || 'primary'
  switch (v) {
    case 'primary':
      return 'bg-accent-purple text-white hover:bg-[#6b4fd8]'
    case 'secondary':
      return 'bg-bg-card text-text-primary border border-border-default hover:bg-bg-input'
    case 'ghost':
      return 'bg-transparent text-text-secondary hover:bg-black/[0.04] hover:text-text-primary'
    default:
      return 'bg-accent-purple text-white'
  }
})
</script>
