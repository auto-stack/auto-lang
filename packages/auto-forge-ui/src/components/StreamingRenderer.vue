<template>
  <div class="streaming-document">
    <template v-for="(segment, idx) in segments" :key="segment.type + '-' + idx">
      <MarkdownRender
        v-if="segment.type === 'markdown'"
        :content="segment.text"
        :final="!streaming"
        :max-live-nodes="streaming ? 0 : 320"
        :batch-rendering="streaming"
        :render-batch-size="16"
        :render-batch-delay="8"
        :typewriter="streaming && idx === lastMarkdownIndex"
        :fade="false"
      />
      <component
        v-else-if="segment.type === 'component'"
        :is="registry[segment.componentType]"
        v-bind="segment.props"
        :final="segment.final"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { MarkdownRender } from 'markstream-vue'
import { useStreamingDocument } from '@/composables/useStreamingDocument'
import StreamingTable from './StreamingTable.vue'

const props = defineProps<{
  source: string
  streaming?: boolean
}>()

const sourceRef = computed(() => props.source)
const { segments } = useStreamingDocument(sourceRef)

const lastMarkdownIndex = computed(() => {
  for (let i = segments.value.length - 1; i >= 0; i--) {
    if (segments.value[i].type === 'markdown') return i
  }
  return -1
})

const registry: Record<string, any> = {
  table: StreamingTable,
  // Future: chart: StreamingChart, form: StreamingForm, ...
}
</script>

<style>
.streaming-document {
  /* markstream-vue already scopes its styles under .markstream-vue */
}

.streaming-document > * + * {
  margin-top: 0.75rem;
}
</style>
