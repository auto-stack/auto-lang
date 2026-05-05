<template>
  <div class="bg-white border border-border-default rounded-lg p-4 shadow-sm">
    <h3 v-if="title" class="text-sm font-semibold text-text-primary mb-3">{{ title }}</h3>
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

const title = computed(() => props.component.title || '')

const childComponent = computed(() => {
  const childId = props.component.child
  if (typeof childId === 'string') {
    return props.allComponents[childId]
  }
  return null
})
</script>
