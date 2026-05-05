<template>
  <div
    class="flex"
    :class="[alignClass, justifyClass]"
    :style="{ gap: `${component.gap || 8}px` }"
  >
    <template v-for="childId in childrenIds" :key="childId">
      <component
        v-if="allComponents[childId]"
        :is="getRenderer(allComponents[childId].component)"
        :component="allComponents[childId]"
        :all-components="allComponents"
        :data-model="dataModel"
      />
    </template>
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

const childrenIds = computed(() => {
  const val = props.component.children
  if (Array.isArray(val)) return val
  if (typeof val === 'string') return [val]
  return []
})

const alignMap: Record<string, string> = {
  start: 'items-start',
  center: 'items-center',
  end: 'items-end',
  stretch: 'items-stretch',
  baseline: 'items-baseline',
}

const justifyMap: Record<string, string> = {
  start: 'justify-start',
  center: 'justify-center',
  end: 'justify-end',
  spaceBetween: 'justify-between',
  spaceAround: 'justify-around',
  spaceEvenly: 'justify-evenly',
}

const alignClass = computed(() => alignMap[props.component.align || 'stretch'] || 'items-stretch')
const justifyClass = computed(() => justifyMap[props.component.justify || 'start'] || 'justify-start')
</script>
