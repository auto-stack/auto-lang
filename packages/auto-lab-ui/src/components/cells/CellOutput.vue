<template>
  <div class="cell-output" :class="status">
    <!-- Diagnostics -->
    <div v-if="output.diagnostics?.length" class="diagnostics-list">
      <div
        v-for="(diag, i) in output.diagnostics"
        :key="i"
        class="diagnostic-item"
        :class="diag.severity"
      >
        <span v-if="diag.line" class="diag-line">Line {{ diag.line }}</span>
        <span class="diag-message">{{ diag.message }}</span>
      </div>
    </div>

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
      <div v-if="output.stderr && !output.diagnostics?.length" class="output-section">
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

.diagnostics-list {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  margin-bottom: 0.5rem;
}

.diagnostic-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.4rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  background: #f38ba811;
  border-left: 3px solid #f38ba8;
}

.diagnostic-item.warning {
  background: #f9e2af11;
  border-left-color: #f9e2af;
}

.diag-line {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  color: #f38ba8;
  white-space: nowrap;
  flex-shrink: 0;
}

.diagnostic-item.warning .diag-line {
  color: #f9e2af;
}

.diag-message {
  color: #cdd6f4;
  word-break: break-word;
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
