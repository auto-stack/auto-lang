<template>
  <div
    class="bg-bg-card border border-border-default rounded-lg"
    :style="{ padding: `${component.padding || 16}px` }"
  >
    <component
      v-if="childComponent"
      :is="getRenderer(childComponent.component)"
      :component="childComponent"
      :all-components="allComponents"
      :data-model="dataModel"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'
import { useRenderer } from './useRenderer'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const { getRenderer } = useRenderer()

const childComponent = computed(() => {
  const childId = props.component.child || props.component.children?.[0]
  if (typeof childId === 'string') {
    return props.allComponents[childId]
  }
  return null
})
</script>
