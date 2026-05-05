<template>
  <div :class="[variantClass, colorClass, alignClass]" class="break-words whitespace-pre-line">
    {{ displayValue }}
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const displayValue = computed(() => {
  return props.component.value || ''
})

const variantMap: Record<string, string> = {
  h1: 'text-3xl font-bold',
  h2: 'text-2xl font-bold',
  h3: 'text-xl font-semibold',
  h4: 'text-lg font-semibold',
  body: 'text-sm',
  caption: 'text-xs',
}

const colorMap: Record<string, string> = {
  'text-primary': 'text-text-primary',
  'text-secondary': 'text-text-secondary',
  'text-muted': 'text-text-muted',
  'accent-purple': 'text-accent-purple',
  'accent-green': 'text-accent-green',
  'accent-amber': 'text-accent-amber',
  'accent-red': 'text-accent-red',
  'accent-blue': 'text-accent-blue',
}

const alignMap: Record<string, string> = {
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
}

const variantClass = computed(() => variantMap[props.component.variant || 'body'] || 'text-sm')
const colorClass = computed(() => colorMap[props.component.color || 'text-primary'] || 'text-text-primary')
const alignClass = computed(() => alignMap[props.component.align || 'left'] || 'text-left')
</script>
