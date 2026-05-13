<template>
  <div class="review-detail">
    <!-- Status summary -->
    <div v-if="summary.total > 0" class="review-summary">
      <div class="summary-chip pass">
        <span class="chip-dot" />
        <span>Pass {{ summary.pass }}</span>
      </div>
      <div class="summary-chip warn">
        <span class="chip-dot" />
        <span>Warning {{ summary.warn }}</span>
      </div>
      <div class="summary-chip fail">
        <span class="chip-dot" />
        <span>Fail {{ summary.fail }}</span>
      </div>
      <div class="summary-chip pending">
        <span class="chip-dot" />
        <span>Pending {{ summary.pending }}</span>
      </div>
    </div>

    <!-- Structured checklist -->
    <div v-if="checklist.length" class="checklist-block">
      <div class="checklist-table">
        <div
          v-for="(item, idx) in checklist"
          :key="idx"
          class="checklist-row"
          :class="item.status"
        >
          <span class="row-status">{{ statusIcon(item.status) }}</span>
          <span class="row-text">{{ item.text }}</span>
          <span v-if="item.note" class="row-note">{{ item.note }}</span>
        </div>
      </div>
    </div>

    <!-- Fallback markdown -->
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

type Status = 'pass' | 'warn' | 'fail' | 'pending'

interface CheckItem {
  text: string
  status: Status
  note: string
}

function detectStatus(text: string): Status {
  if (text.includes('✅') || text.includes('✓') || text.includes('Pass') || text.includes('pass')) return 'pass'
  if (text.includes('⚠️') || text.includes('❗') || text.includes('Warning') || text.includes('warn')) return 'warn'
  if (text.includes('❌') || text.includes('✗') || text.includes('Fail') || text.includes('fail')) return 'fail'
  return 'pending'
}

function statusIcon(status: Status): string {
  const map: Record<Status, string> = { pass: '✓', warn: '!', fail: '✕', pending: '○' }
  return map[status]
}

const checklist = computed<CheckItem[]>(() => {
  const items: CheckItem[] = []
  const lines = props.content.split('\n')
  for (const raw of lines) {
    const line = raw.trim()
    // Match list items with status emoji
    const m = line.match(/^[-*]\s+([✅✓⚠️❗❌✗⏳○]\s+)?(.+)$/)
    if (m) {
      const status = detectStatus(line)
      const text = m[2].trim()
      // Try to extract note after dash or semicolon
      const noteMatch = text.match(/^(.+?)\s*(?:[-—]\s*(.+))?$/)
      items.push({
        text: noteMatch ? noteMatch[1].trim() : text,
        status,
        note: noteMatch && noteMatch[2] ? noteMatch[2].trim() : '',
      })
    }
  }
  return items
})

const summary = computed(() => {
  const s = { pass: 0, warn: 0, fail: 0, pending: 0, total: 0 }
  checklist.value.forEach(i => {
    s[i.status]++
    s.total++
  })
  return s
})

const processedContent = computed(() => {
  // Remove checklist list items that we've parsed, keep tables and other content
  const lines = props.content.split('\n')
  const kept: string[] = []
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.match(/^[-*]\s+[✅✓⚠️❗❌✗⏳○]/)) continue
    kept.push(line)
  }
  return kept.join('\n')
})
</script>

<style scoped>
.review-detail {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.review-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.summary-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.2rem 0.5rem;
  font-size: 0.72rem;
  font-weight: 600;
  border-radius: 6px;
  border: 1px solid transparent;
}

.summary-chip.pass {
  background: hsl(142 71% 45% / 0.1);
  color: hsl(142 71% 35%);
  border-color: hsl(142 71% 45% / 0.25);
}

.summary-chip.warn {
  background: hsl(38 92% 50% / 0.1);
  color: hsl(38 92% 40%);
  border-color: hsl(38 92% 50% / 0.25);
}

.summary-chip.fail {
  background: hsl(0 72% 51% / 0.1);
  color: hsl(0 72% 45%);
  border-color: hsl(0 72% 51% / 0.25);
}

.summary-chip.pending {
  background: hsl(var(--muted-foreground) / 0.06);
  color: var(--af-muted);
  border-color: var(--af-border);
}

.chip-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.checklist-block {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow: hidden;
}

.checklist-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.82rem;
  border-bottom: 1px solid var(--af-border);
  transition: background 0.12s;
}

.checklist-row:last-child {
  border-bottom: none;
}

.checklist-row:hover {
  background: hsl(var(--muted-foreground) / 0.02);
}

.row-status {
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-size: 0.65rem;
  font-weight: 700;
  flex-shrink: 0;
}

.checklist-row.pass .row-status {
  background: hsl(142 71% 45% / 0.12);
  color: hsl(142 71% 35%);
}

.checklist-row.warn .row-status {
  background: hsl(38 92% 50% / 0.12);
  color: hsl(38 92% 40%);
}

.checklist-row.fail .row-status {
  background: hsl(0 72% 51% / 0.12);
  color: hsl(0 72% 45%);
}

.checklist-row.pending .row-status {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-muted);
}

.row-text {
  flex: 1;
  color: var(--af-fg);
}

.row-note {
  font-size: 0.75rem;
  color: var(--af-muted);
  max-width: 200px;
  text-align: right;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
