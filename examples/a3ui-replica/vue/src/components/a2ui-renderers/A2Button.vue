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
      return 'bg-violet-600 text-white hover:bg-violet-700'
    case 'secondary':
      return 'bg-white text-slate-700 border border-slate-200 hover:bg-slate-50'
    case 'ghost':
      return 'bg-transparent text-slate-500 hover:bg-slate-100 hover:text-slate-700'
    default:
      return 'bg-violet-600 text-white hover:bg-violet-700'
  }
})
</script>
