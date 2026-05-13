<template>
  <div class="test-detail">
    <!-- Meta badges -->
    <div v-if="meta.type || meta.scope || meta.testFile" class="test-meta">
      <span v-if="meta.type" class="meta-badge type">{{ meta.type }}</span>
      <span v-if="meta.scope" class="meta-badge scope">Scope: {{ meta.scope }}</span>
      <span v-if="meta.testFile" class="meta-badge file">📄 {{ meta.testFile }}</span>
    </div>

    <!-- Fixture -->
    <div v-if="meta.fixture" class="fixture-block">
      <div class="block-label">Fixture</div>
      <pre class="fixture-code"><code>{{ meta.fixture }}</code></pre>
    </div>

    <!-- Steps -->
    <div v-if="meta.steps.length" class="steps-block">
      <div class="block-label">Steps</div>
      <ol class="steps-list">
        <li v-for="(step, idx) in meta.steps" :key="idx">{{ step }}</li>
      </ol>
    </div>

    <!-- Expected Outcome -->
    <div v-if="meta.expected" class="expected-block">
      <div class="block-label">Expected Outcome</div>
      <div class="expected-body">{{ meta.expected }}</div>
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
  testFile?: string
}>()

const emit = defineEmits<{
  linkClick: [id: string]
}>()

const meta = computed(() => {
  const c = props.content
  const type = c.match(/\*\*Type:\*\*\s*(\w+)/)?.[1] || ''
  const scope = c.match(/\*\*Scope:\*\*\s*(.+)/)?.[1] || ''
  const fixture = extractBlock(c, 'Fixture')
  const expected = extractBlock(c, 'Expected Outcome')
  const steps = extractSteps(c)
  const testFile = props.testFile || c.match(/\*\*Test File:\*\*\s*(.+)/)?.[1] || ''
  return { type, scope, fixture, expected, steps, testFile }
})

const remainingContent = computed(() => {
  // Return content that isn't part of the structured fields
  const lines = props.content.split('\n')
  const kept: string[] = []
  let skipUntil = ''
  let skipping = false
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.match(/^\*\*Type:\*\*/)) continue
    if (line.match(/^\*\*Scope:\*\*/)) continue
    if (line.match(/^\*\*Test File:\*\*/)) continue
    if (line.match(/^\*\*Fixture:\*\*/)) {
      skipping = true
      skipUntil = ''
      continue
    }
    if (line.match(/^\*\*Expected Outcome:\*\*/)) {
      skipping = true
      skipUntil = ''
      continue
    }
    if (skipping && line.match(/^\*\*/)) {
      skipping = false
    }
    if (skipping && line.trim() === '```') {
      skipping = false
      continue
    }
    if (skipping) continue
    if (line.match(/^\d+\.\s+/)) continue
    if (line.trim()) kept.push(line)
  }
  return kept.join('\n')
})

function extractBlock(content: string, label: string): string {
  const re = new RegExp(`\\*\\*${label}:\\*\\*\\s*\\n?([\\s\\S]*?)(?=\\n\\*\\*|$)`, 'i')
  const m = content.match(re)
  if (!m) return ''
  let block = m[1].trim()
  // Remove surrounding ``` if present
  block = block.replace(/^```[\w]*\n?/, '').replace(/\n?```$/, '')
  return block.trim()
}

function extractSteps(content: string): string[] {
  const steps: string[] = []
  const re = /^\d+\.\s*(.*)$/gm
  let m: RegExpExecArray | null
  while ((m = re.exec(content)) !== null) {
    steps.push(m[1].trim())
  }
  return steps
}
</script>

<style scoped>
.test-detail {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.test-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.meta-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.15rem 0.5rem;
  font-size: 0.72rem;
  font-weight: 600;
  border-radius: 6px;
  border: 1px solid var(--af-border);
}

.meta-badge.type {
  background: hsl(217 91% 60% / 0.1);
  color: #3b82f6;
  border-color: hsl(217 91% 60% / 0.25);
}

.meta-badge.scope {
  background: hsl(38 92% 50% / 0.1);
  color: #f59e0b;
  border-color: hsl(38 92% 50% / 0.25);
}

.meta-badge.file {
  background: hsl(var(--muted-foreground) / 0.06);
  color: var(--af-muted);
}

.block-label {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--af-muted);
  margin-bottom: 0.3rem;
}

.fixture-block {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow: hidden;
}

.fixture-code {
  margin: 0;
  padding: 0.6rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.04);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.8rem;
  line-height: 1.5;
  color: var(--af-fg);
  overflow-x: auto;
}

.steps-block {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.6rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.02);
}

.steps-list {
  margin: 0;
  padding-left: 1.2rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.steps-list li {
  font-size: 0.82rem;
  color: var(--af-fg);
  line-height: 1.5;
  padding-left: 0.3rem;
}

.steps-list li::marker {
  color: hsl(var(--primary));
  font-weight: 600;
}

.expected-block {
  border-left: 3px solid hsl(142 71% 45%);
  padding: 0.5rem 0.75rem;
  background: hsl(142 71% 45% / 0.05);
  border-radius: 0 8px 8px 0;
}

.expected-body {
  font-size: 0.82rem;
  color: var(--af-fg);
  line-height: 1.5;
  white-space: pre-wrap;
}
</style>
