<template>
  <div class="flex flex-col">
    <span class="text-xs text-text-secondary mb-1">{{ component.label || '' }}</span>
    <div class="flex items-baseline gap-2">
      <span class="text-2xl font-bold text-text-primary">{{ component.value || '' }}</span>
      <span
        v-if="trendIcon"
        class="text-xs"
        :class="trendColor"
      >
        {{ trendIcon }}
      </span>
    </div>
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

const trendIcon = computed(() => {
  const t = props.component.trend
  if (t === 'up') return '↑'
  if (t === 'down') return '↓'
  return null
})

const trendColor = computed(() => {
  const t = props.component.trend
  if (t === 'up') return 'text-accent-green'
  if (t === 'down') return 'text-accent-red'
  return 'text-text-muted'
})
</script>
