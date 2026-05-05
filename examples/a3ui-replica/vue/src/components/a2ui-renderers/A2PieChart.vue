<template>
  <div class="flex items-center justify-center">
    <svg :width="size" :height="size" viewBox="-1.1 -1.1 2.2 2.2">
      <circle
        v-for="(slice, i) in slices"
        :key="i"
        r="1"
        cx="0"
        cy="0"
        fill="none"
        :stroke="slice.color"
        :stroke-width="0.3"
        :stroke-dasharray="`${slice.length} ${2 * Math.PI - slice.length}`"
        :transform="`rotate(${slice.rotate})`"
      />
      <circle r="0.6" cx="0" cy="0" fill="white" />
    </svg>
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

const size = 120
const colors = ['#7c5cfc', '#10b981', '#f59e0b', '#ef4444', '#3b82f6', '#8b5cf6']

const slices = computed(() => {
  const data = props.component.data || []
  const total = data.reduce((sum: number, d: any) => sum + (d.value || 0), 0)
  let accumulated = 0

  return data.map((d: any, i: number) => {
    const value = d.value || 0
    const length = total > 0 ? (value / total) * 2 * Math.PI : 0
    const rotate = accumulated * -360 // Rotate counter-clockwise
    accumulated += value / total
    return {
      color: colors[i % colors.length],
      length,
      rotate,
    }
  })
})
</script>
