<template>
  <div class="api-detail">
    <!-- Endpoint cards -->
    <div v-if="endpoints.length" class="endpoints-section">
      <div class="section-label">Endpoints</div>
      <div class="endpoint-list">
        <div
          v-for="(ep, idx) in endpoints"
          :key="idx"
          class="endpoint-card"
        >
          <div class="endpoint-main">
            <span class="method-badge" :class="ep.method.toLowerCase()">{{ ep.method }}</span>
            <code class="endpoint-path">{{ ep.path }}</code>
          </div>
          <div v-if="ep.description" class="endpoint-desc">{{ ep.description }}</div>
        </div>
      </div>
    </div>

    <!-- Fallback markdown for schemas, auth, etc. -->
    <MarkdownContent :content="remainingContent" @link-click="$emit('linkClick', $event)" />
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

interface Endpoint {
  method: string
  path: string
  description: string
}

const endpoints = computed<Endpoint[]>(() => {
  const eps: Endpoint[] = []
  const lines = props.content.split('\n')
  let inTable = false
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.includes('| Method |') || line.includes('| method |')) {
      inTable = true
      continue
    }
    if (inTable && line.startsWith('|')) {
      const cells = line.split('|').map(c => c.trim()).filter(Boolean)
      if (cells.length >= 3) {
        const method = cells[0].toUpperCase()
        if (['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'].includes(method)) {
          eps.push({
            method,
            path: cells[1],
            description: cells[2] || '',
          })
        }
      }
    } else if (inTable && !line.startsWith('|') && line.trim()) {
      inTable = false
    }
  }
  return eps
})

const remainingContent = computed(() => {
  const lines = props.content.split('\n')
  const kept: string[] = []
  let inTable = false
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.includes('| Method |') || line.includes('| method |')) {
      inTable = true
      continue
    }
    if (inTable && !line.startsWith('|') && line.trim()) {
      inTable = false
    }
    if (inTable) continue
    kept.push(line)
  }
  return kept.join('\n')
})
</script>

<style scoped>
.api-detail {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.section-label {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--af-muted);
  margin-bottom: 0.2rem;
}

.endpoint-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.endpoint-card {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.6rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.02);
  transition: background 0.12s;
}

.endpoint-card:hover {
  background: hsl(var(--muted-foreground) / 0.04);
}

.endpoint-main {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.method-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.1rem 0.4rem;
  font-size: 0.65rem;
  font-weight: 700;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  min-width: 42px;
}

.method-badge.get {
  background: hsl(217 91% 60% / 0.12);
  color: #3b82f6;
}

.method-badge.post {
  background: hsl(142 71% 45% / 0.12);
  color: hsl(142 71% 35%);
}

.method-badge.put,
.method-badge.patch {
  background: hsl(38 92% 50% / 0.12);
  color: #f59e0b;
}

.method-badge.delete {
  background: hsl(0 72% 51% / 0.12);
  color: #ef4444;
}

.method-badge.head,
.method-badge.options {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-muted);
}

.endpoint-path {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.82rem;
  color: var(--af-fg);
  background: none;
}

.endpoint-desc {
  margin-top: 0.25rem;
  font-size: 0.78rem;
  color: var(--af-muted);
  padding-left: calc(42px + 0.5rem);
}
</style>
