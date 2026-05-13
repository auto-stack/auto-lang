<template>
  <div class="report-detail">
    <!-- Metrics cards -->
    <div v-if="metrics.length" class="metrics-grid">
      <div
        v-for="(m, idx) in metrics"
        :key="idx"
        class="metric-card"
      >
        <div class="metric-name">{{ m.name }}</div>
        <div class="metric-value" :class="m.status">{{ m.value }}</div>
        <div v-if="m.target" class="metric-target">Target: {{ m.target }}</div>
      </div>
    </div>

    <!-- Coverage / benchmark tables rendered by markdown -->
    <MarkdownContent :content="processedContent" @link-click="$emit('linkClick', $event)" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MarkdownContent from '@/components/MarkdownContent.vue'

const props = defineProps<{
  content: string
}>()

const emit = defineEmits<{
  linkClick: [id: string]
}>()

interface Metric {
  name: string
  value: string
  target: string
  status: 'good' | 'warn' | 'bad' | 'neutral'
}

function parseValue(value: string): { num: number | null; unit: string } {
  const m = value.match(/^([\d.]+)\s*(.*)$/)
  if (!m) return { num: null, unit: value }
  return { num: parseFloat(m[1]), unit: m[2] }
}

function compareStatus(value: string, target: string): Metric['status'] {
  const v = parseValue(value)
  const t = parseValue(target)
  if (v.num === null || t.num === null) return 'neutral'
  // Heuristic: if target contains < or >, parse direction
  if (target.includes('<')) {
    return v.num < t.num ? 'good' : 'bad'
  }
  if (target.includes('>')) {
    return v.num > t.num ? 'good' : 'bad'
  }
  // Percentage-style comparison
  const ratio = v.num / t.num
  if (ratio >= 1) return 'good'
  if (ratio >= 0.7) return 'warn'
  return 'bad'
}

const metrics = computed<Metric[]>(() => {
  const list: Metric[] = []
  // Parse markdown tables with | Metric | Score | Target |
  const lines = props.content.split('\n')
  let inMetricsTable = false
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.match(/^\|?\s*Metric\s*\|\s*Score\s*\|\s*Target\s*\|?/i)) {
      inMetricsTable = true
      continue
    }
    if (inMetricsTable && line.startsWith('|')) {
      const cells = line.split('|').map(c => c.trim()).filter(Boolean)
      if (cells.length >= 2 && cells[0] !== 'Metric' && cells[0] !== '---') {
        const name = cells[0]
        const value = cells[1] === '—' || cells[1] === '-' ? '' : cells[1]
        const target = cells[2] === '—' || cells[2] === '-' ? '' : cells[2] || ''
        list.push({
          name,
          value: value || '—',
          target,
          status: value ? compareStatus(value, target) : 'neutral',
        })
      }
    } else if (inMetricsTable && line.trim() && !line.startsWith('|')) {
      inMetricsTable = false
    }
  }
  return list
})

const processedContent = computed(() => {
  // Keep tables but metrics table is already rendered as cards
  const lines = props.content.split('\n')
  const kept: string[] = []
  let inMetricsTable = false
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.match(/^\|?\s*Metric\s*\|\s*Score\s*\|\s*Target\s*\|?/i)) {
      inMetricsTable = true
      continue
    }
    if (inMetricsTable && line.trim() && !line.startsWith('|')) {
      inMetricsTable = false
    }
    if (inMetricsTable) continue
    kept.push(line)
  }
  return kept.join('\n')
})
</script>

<style scoped>
.report-detail {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 0.5rem;
}

.metric-card {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.6rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.02);
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.metric-name {
  font-size: 0.72rem;
  color: var(--af-muted);
  font-weight: 500;
}

.metric-value {
  font-size: 1.1rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: var(--af-fg);
}

.metric-value.good {
  color: hsl(142 71% 35%);
}

.metric-value.warn {
  color: hsl(38 92% 40%);
}

.metric-value.bad {
  color: hsl(0 72% 45%);
}

.metric-target {
  font-size: 0.7rem;
  color: var(--af-muted);
}
</style>
