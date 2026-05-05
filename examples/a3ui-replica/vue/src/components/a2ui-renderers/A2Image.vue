<template>
  <img
    :src="component.src"
    :alt="component.alt || ''"
    class="object-cover"
    :class="radiusClass"
    :style="{ width: component.width ? `${component.width}px` : '100%', height: component.height ? `${component.height}px` : 'auto' }"
    @error="onError"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const radiusMap: Record<string, string> = {
  none: 'rounded-none',
  sm: 'rounded-sm',
  md: 'rounded-md',
  lg: 'rounded-lg',
  full: 'rounded-full',
}

const radiusClass = computed(() => radiusMap[props.component.borderRadius || 'md'] || 'rounded-md')

function onError(e: Event) {
  const target = e.target as HTMLImageElement
  target.style.display = 'none'
}
</script>
