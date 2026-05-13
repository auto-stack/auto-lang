<template>
  <div class="goal-detail">
    <!-- Acceptance Criteria -->
    <div v-if="criteria.length" class="criteria-block">
      <div class="block-label">Acceptance Criteria</div>
      <ul class="criteria-list">
        <li
          v-for="(c, idx) in criteria"
          :key="idx"
          :class="{ checked: c.checked }"
        >
          <span class="criteria-check">{{ c.checked ? '☑' : '☐' }}</span>
          <span class="criteria-text">{{ c.text }}</span>
        </li>
      </ul>
    </div>

    <!-- Details -->
    <div v-if="details" class="details-block">
      <div class="block-label">Details</div>
      <div class="details-body">{{ details }}</div>
    </div>

    <!-- Fallback markdown -->
    <MarkdownContent v-if="remainingContent" :content="remainingContent" @link-click="$emit('linkClick', $event)" />
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

const criteria = computed(() => {
  const list: { text: string; checked: boolean }[] = []
  const lines = props.content.split('\n')
  let inCriteria = false
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.startsWith('**Acceptance Criteria:**')) {
      inCriteria = true
      continue
    }
    if (line.startsWith('**Details:**') || line.startsWith('## ') || line.startsWith('# ')) {
      inCriteria = false
      continue
    }
    if (inCriteria) {
      const m = line.match(/^-\s+\[([ xX])\]\s*(.*)$/)
      if (m) {
        list.push({ text: m[2].trim(), checked: m[1].toLowerCase() === 'x' })
      }
    }
  }
  return list
})

const details = computed(() => {
  const lines = props.content.split('\n')
  let inDetails = false
  const detailLines: string[] = []
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.startsWith('**Details:**')) {
      inDetails = true
      continue
    }
    if (inDetails && (line.startsWith('**') || line.startsWith('## ') || line.startsWith('# '))) {
      inDetails = false
      continue
    }
    if (inDetails) detailLines.push(line)
  }
  return detailLines.join('\n').trim()
})

const remainingContent = computed(() => {
  const lines = props.content.split('\n')
  const kept: string[] = []
  let skipSection = ''
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.startsWith('**Acceptance Criteria:**')) { skipSection = 'criteria'; continue }
    if (line.startsWith('**Details:**')) { skipSection = 'details'; continue }
    if (skipSection && line.startsWith('**')) skipSection = ''
    if (skipSection === 'criteria' && line.match(/^-\s+\[[ xX]\]/)) continue
    if (skipSection === 'details') continue
    if (line.trim()) kept.push(line)
  }
  return kept.join('\n')
})
</script>

<style scoped>
.goal-detail {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.block-label {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--af-muted);
  margin-bottom: 0.3rem;
}

.criteria-block {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.6rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.02);
}

.criteria-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.criteria-list li {
  display: flex;
  align-items: flex-start;
  gap: 0.4rem;
  font-size: 0.82rem;
  color: var(--af-fg);
  line-height: 1.4;
}

.criteria-list li.checked {
  opacity: 0.55;
}

.criteria-list li.checked .criteria-text {
  text-decoration: line-through;
}

.criteria-check {
  font-size: 0.9rem;
  color: hsl(var(--primary));
  min-width: 1rem;
  text-align: center;
}

.criteria-list li.checked .criteria-check {
  color: hsl(142 71% 45%);
}

.criteria-text {
  flex: 1;
}

.details-block {
  border-left: 3px solid hsl(var(--primary));
  padding: 0.5rem 0.75rem;
  background: hsl(var(--primary) / 0.04);
  border-radius: 0 8px 8px 0;
}

.details-body {
  font-size: 0.82rem;
  color: var(--af-fg);
  line-height: 1.6;
  white-space: pre-wrap;
}
</style>
