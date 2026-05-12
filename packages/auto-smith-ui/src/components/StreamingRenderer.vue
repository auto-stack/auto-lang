<template>
  <div class="streaming-document">
    <MarkdownRender
      :content="cleanText"
      :final="!streaming"
      :max-live-nodes="streaming ? 0 : 320"
      :batch-rendering="streaming"
      :render-batch-size="16"
      :render-batch-delay="8"
      :typewriter="streaming"
      :fade="false"
    />
    <div
      v-for="node in nodes"
      :key="node.id"
      class="streaming-node"
    >
      <component
        :is="registry[node.type]"
        v-bind="node.props"
        :final="node.final"
      />
    </div>
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
const { cleanText, nodes } = useStreamingDocument(sourceRef)

const registry: Record<string, any> = {
  table: StreamingTable,
  // Future: chart: StreamingChart, form: StreamingForm, ...
}
</script>

<style>
.streaming-document {
  /* markstream-vue already scopes its styles under .markstream-vue */
}

.streaming-node {
  margin-top: 0.75rem;
}
</style>
