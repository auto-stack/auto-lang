<template>
  <div class="report-card" :class="{ collapsed: !expanded }">
    <div class="report-header" @click="expanded = !expanded">
      <span class="report-status">✅</span>
      <span class="report-title">Relay Complete — Report {{ report.runId }}</span>
      <span class="report-confidence" :class="report.confidence.toLowerCase()">
        {{ report.confidence }}
      </span>
      <ChevronDown v-if="!expanded" :size="14" class="report-chevron" />
      <ChevronUp v-else :size="14" class="report-chevron" />
    </div>

    <div v-if="expanded" class="report-body">
      <div class="report-metrics">
        <div class="metric-row">
          <span class="metric-label">Goals Met</span>
          <span class="metric-value">{{ report.goalsMet }}</span>
        </div>
        <div class="metric-row">
          <span class="metric-label">Tests Pass</span>
          <span class="metric-value">{{ report.testsPass }}</span>
        </div>
        <div class="metric-row">
          <span class="metric-label">Drift Detected</span>
          <span class="metric-value" :class="{ drift: report.driftDetected !== 'None' }">
            {{ report.driftDetected }}
          </span>
        </div>
        <div class="metric-row">
          <span class="metric-label">Cost</span>
          <span class="metric-value">{{ report.cost }}</span>
        </div>
      </div>

      <div v-if="report.deliverables.length > 0" class="report-deliverables">
        <div class="section-title">Deliverables</div>
        <ul>
          <li v-for="(d, i) in report.deliverables" :key="i">{{ d }}</li>
        </ul>
      </div>

      <div class="report-actions">
        <button class="report-btn" @click.stop="$emit('view-full')">
          <FileText :size="13" />
          View Full Report
        </button>
        <button class="report-btn" @click.stop="$emit('download')">
          <Download :size="13" />
          Download Markdown
        </button>
        <button class="report-btn" @click.stop="$emit('open-files')">
          <FolderOpen :size="13" />
          Open Changed Files
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ChevronDown, ChevronUp, FileText, Download, FolderOpen } from 'lucide-vue-next'

export interface ReportData {
  runId: string
  goalsMet: string
  testsPass: string
  driftDetected: string
  cost: string
  confidence: 'High' | 'Medium' | 'Low'
  deliverables: string[]
}

interface Props {
  report: ReportData
}

defineProps<Props>()

defineEmits<{
  (e: 'view-full'): void
  (e: 'download'): void
  (e: 'open-files'): void
}>()

const expanded = ref(false)
</script>

<style scoped>
.report-card {
  border: 1px solid hsl(142 70% 45% / 0.25);
  border-radius: 10px;
  background: hsl(142 70% 45% / 0.04);
  margin: 0.5rem 0;
  overflow: hidden;
  transition: all 0.2s;
}

.report-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.6rem 0.8rem;
  cursor: pointer;
  user-select: none;
}

.report-header:hover {
  background: hsl(142 70% 45% / 0.06);
}

.report-status {
  font-size: 1rem;
  flex-shrink: 0;
}

.report-title {
  flex: 1;
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--af-fg);
}

.report-confidence {
  font-size: 0.65rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  font-weight: 500;
  text-transform: uppercase;
}

.report-confidence.high {
  background: hsl(142 70% 45% / 0.15);
  color: hsl(142 70% 35%);
}

.report-confidence.medium {
  background: hsl(38 90% 50% / 0.15);
  color: hsl(38 80% 40%);
}

.report-confidence.low {
  background: hsl(0 70% 50% / 0.15);
  color: hsl(0 70% 45%);
}

.report-chevron {
  color: var(--af-muted);
  flex-shrink: 0;
}

.report-body {
  padding: 0.5rem 0.8rem 0.75rem;
  border-top: 1px solid hsl(142 70% 45% / 0.15);
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.report-metrics {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.4rem;
}

.metric-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.3rem 0.4rem;
  background: hsl(var(--muted-foreground) / 0.04);
  border-radius: 5px;
  font-size: 0.75rem;
}

.metric-label {
  color: var(--af-muted);
}

.metric-value {
  font-weight: 500;
  color: var(--af-fg);
}

.metric-value.drift {
  color: hsl(var(--af-error));
}

.section-title {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--af-muted);
  letter-spacing: 0.03em;
  margin-bottom: 0.2rem;
}

.report-deliverables ul {
  margin: 0;
  padding-left: 1.1rem;
  font-size: 0.8rem;
  color: var(--af-fg);
  line-height: 1.5;
}

.report-deliverables li {
  margin: 0.1rem 0;
}

.report-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.report-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.35rem 0.6rem;
  border: 1px solid var(--af-border);
  border-radius: 5px;
  background: transparent;
  color: var(--af-fg);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.report-btn:hover {
  background: hsl(var(--muted-foreground) / 0.06);
  border-color: hsl(var(--primary) / 0.3);
}
</style>
