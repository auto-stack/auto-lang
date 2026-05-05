<template>
  <div class="a2ui-renderer w-full h-full">
    <component
      v-if="rootComponent"
      :is="getRenderer(rootComponent.component)"
      :component="rootComponent"
      :all-components="componentMap"
      :data-model="dataModel"
    />
    <div v-else class="flex items-center justify-center h-full text-text-muted text-sm">
      No components to render
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'
import A2Row from './a2ui-renderers/A2Row.vue'
import A2Column from './a2ui-renderers/A2Column.vue'
import A2List from './a2ui-renderers/A2List.vue'
import A2Card from './a2ui-renderers/A2Card.vue'
import A2Text from './a2ui-renderers/A2Text.vue'
import A2Image from './a2ui-renderers/A2Image.vue'
import A2Icon from './a2ui-renderers/A2Icon.vue'
import A2Button from './a2ui-renderers/A2Button.vue'
import A2Tabs from './a2ui-renderers/A2Tabs.vue'
import A2TextField from './a2ui-renderers/A2TextField.vue'
import A2CheckBox from './a2ui-renderers/A2CheckBox.vue'
import A2Slider from './a2ui-renderers/A2Slider.vue'
import A2Divider from './a2ui-renderers/A2Divider.vue'
import A2DashboardCard from './a2ui-renderers/A2DashboardCard.vue'
import A2Title from './a2ui-renderers/A2Title.vue'
import A2Metric from './a2ui-renderers/A2Metric.vue'
import A2Badge from './a2ui-renderers/A2Badge.vue'
import A2PieChart from './a2ui-renderers/A2PieChart.vue'
import A2BarChart from './a2ui-renderers/A2BarChart.vue'
import A2DataTable from './a2ui-renderers/A2DataTable.vue'
import { useRenderer } from './a2ui-renderers/useRenderer'

const props = defineProps<{
  components: A2UIComponent[]
  dataModel?: Record<string, any>
}>()

const { getRenderer } = useRenderer()

const componentMap = computed(() => {
  const map: Record<string, A2UIComponent> = {}
  props.components.forEach(c => { map[c.id] = c })
  return map
})

const rootComponent = computed(() => {
  return props.components.find(c => c.id === 'root') || props.components[0]
})
</script>
