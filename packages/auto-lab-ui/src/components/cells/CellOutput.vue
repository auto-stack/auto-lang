<template>
  <div class="cell-output" :class="status">
    <!-- Rich output for chart/table types -->
    <OutputChart v-if="cellType === 'chart'" :source="output.result || output.stdout" />
    <OutputTable v-else-if="cellType === 'table'" :source="output.result || output.stdout" />

    <!-- Standard text output for code/ai/markdown -->
    <template v-else>
      <div v-if="output.stdout" class="output-section">
        <div class="output-label">stdout</div>
        <pre class="output-text stdout">{{ output.stdout }}</pre>
      </div>
      <div v-if="output.result" class="output-section">
        <div class="output-label">result</div>
        <pre class="output-text result">{{ output.result }}</pre>
      </div>
      <div v-if="output.stderr" class="output-section">
        <div class="output-label">stderr</div>
        <pre class="output-text stderr">{{ output.stderr }}</pre>
      </div>
    </template>

    <div class="output-meta">
      <span class="time-badge">{{ output.time_ms }}ms</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import OutputChart from './OutputChart.vue'
import OutputTable from './OutputTable.vue'
import type { CellOutput, CellStatus, CellType } from '@/types/cell'

defineProps<{
  output: CellOutput
  status: CellStatus
  cellType?: CellType
}>()
</script>

<style scoped>
.cell-output {
  border-top: 1px solid #313244;
  background: #181825;
  padding: 0.5rem 0.75rem;
  font-size: 0.85rem;
}

.output-section {
  margin-bottom: 0.5rem;
}

.output-section:last-child {
  margin-bottom: 0;
}

.output-label {
  font-size: 0.7rem;
  text-transform: uppercase;
  color: #6c7086;
  margin-bottom: 0.2rem;
  font-weight: 600;
  letter-spacing: 0.03em;
}

.output-text {
  margin: 0;
  padding: 0.4rem 0.5rem;
  border-radius: 4px;
  background: #1e1e2e;
  color: #cdd6f4;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-x: auto;
}

.output-text.stderr {
  background: #f38ba811;
  color: #f38ba8;
}

.output-text.result {
  background: #27c93f11;
  color: #27c93f;
}

.output-meta {
  display: flex;
  justify-content: flex-end;
  margin-top: 0.35rem;
}

.time-badge {
  font-size: 0.7rem;
  color: #6c7086;
  font-family: 'JetBrains Mono', monospace;
}
</style>
