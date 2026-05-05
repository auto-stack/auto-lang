<template>
  <span
    class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
    :class="variantClass"
  >
    {{ component.label || '' }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const variantMap: Record<string, string> = {
  success: 'bg-green-100 text-green-800',
  error: 'bg-red-100 text-red-800',
  warning: 'bg-yellow-100 text-yellow-800',
  info: 'bg-blue-100 text-blue-800',
  neutral: 'bg-gray-100 text-gray-800',
}

const variantClass = computed(() => variantMap[props.component.variant || 'neutral'] || 'bg-gray-100 text-gray-800')
</script>
