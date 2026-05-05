<template>
  <div :class="[variantClass]" class="text-text-primary">
    {{ component.text || '' }}
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

const variantMap: Record<string, string> = {
  h1: 'text-3xl font-bold',
  h2: 'text-2xl font-bold',
  h3: 'text-xl font-semibold',
  h4: 'text-lg font-semibold',
  body: 'text-sm',
  caption: 'text-xs',
}

const variantClass = computed(() => variantMap[props.component.variant || 'h3'] || 'text-xl font-semibold')
</script>
