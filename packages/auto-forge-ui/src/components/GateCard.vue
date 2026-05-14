<template>
  <div class="gate-card">
    <div class="gate-card-header">
      <span class="gate-icon">🔒</span>
      <span class="gate-title">{{ title }}</span>
      <span v-if="profession" class="gate-profession">{{ profession }}</span>
    </div>

    <div v-if="changes.length > 0" class="gate-diff-list">
      <div
        v-for="change in changes"
        :key="change.section_id"
        class="diff-card"
      >
        <div class="diff-header" @click="toggleDiff(change.section_id)">
          <span class="diff-title">{{ change.section_id }}</span>
          <span class="diff-status" :class="change.new_status">
            {{ change.old_status }} → {{ change.new_status }}
          </span>
          <ChevronDown v-if="!expandedDiffs.has(change.section_id)" :size="14" class="diff-chevron" />
          <ChevronUp v-else :size="14" class="diff-chevron" />
        </div>
        <div v-if="expandedDiffs.has(change.section_id)" class="diff-body">
          <div class="diff-side">
            <div class="diff-label">Before</div>
            <pre class="diff-content old">{{ change.old_content }}</pre>
          </div>
          <div class="diff-side">
            <div class="diff-label">After</div>
            <textarea
              v-model="editedSpecs[change.section_id]"
              class="diff-editor"
              rows="6"
            />
          </div>
        </div>
      </div>
    </div>

    <div class="gate-actions">
      <button ref="approveBtnRef" class="approve-btn" @click="$emit('approve', editedSpecs)">
        <Check :size="14" />
        Approve & Execute
      </button>
      <button class="reject-btn" @click="$emit('reject')">
        <X :size="14" />
        Reject & Redraft
      </button>
      <button
        class="review-btn"
        @click="isReviewing = !isReviewing"
      >
        <Eye :size="14" />
        {{ isReviewing ? 'Hide' : 'Review' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
import { Check, X, Eye, ChevronDown, ChevronUp } from 'lucide-vue-next'
import type { SpecChange } from '@/types/forge'

interface Props {
  title?: string
  profession?: string
  changes: SpecChange[]
}

const props = withDefaults(defineProps<Props>(), {
  title: 'Spec drafted. Review the proposed changes below.',
  profession: undefined,
})

defineEmits<{
  (e: 'approve', editedSpecs: Record<string, string>): void
  (e: 'reject'): void
}>()

const expandedDiffs = ref<Set<string>>(new Set())
const editedSpecs = ref<Record<string, string>>({})
const isReviewing = ref(false)
const approveBtnRef = ref<HTMLButtonElement | null>(null)

onMounted(() => {
  // Focus management: auto-focus approve button for keyboard users
  nextTick(() => {
    approveBtnRef.value?.focus()
  })
})

function toggleDiff(sectionId: string) {
  if (expandedDiffs.value.has(sectionId)) {
    expandedDiffs.value.delete(sectionId)
  } else {
    expandedDiffs.value.add(sectionId)
  }
}

watch(() => props.changes, (changes) => {
  for (const change of changes) {
    if (!(change.section_id in editedSpecs.value)) {
      editedSpecs.value[change.section_id] = change.new_content
    }
  }
}, { immediate: true, deep: true })
</script>

<style scoped>
.gate-card {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--af-border);
  flex-shrink: 0;
  background: hsl(var(--muted-foreground) / 0.02);
}

.gate-card-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  color: var(--af-fg);
}

.gate-icon { font-size: 1.1rem; }
.gate-title { flex: 1; }
.gate-profession {
  font-size: 0.7rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  background: hsl(var(--primary) / 0.08);
  color: var(--af-primary);
  font-weight: 500;
}

.gate-diff-list {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  max-height: 300px;
  overflow-y: auto;
}

.diff-card {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow: hidden;
}

.diff-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  cursor: pointer;
  user-select: none;
}

.diff-header:hover {
  background: hsl(var(--muted-foreground) / 0.03);
}

.diff-title {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--af-fg);
  text-transform: capitalize;
  flex: 1;
}

.diff-status {
  font-size: 0.65rem;
  font-weight: 500;
  color: hsl(var(--af-warning));
}

.diff-status.approved { color: hsl(var(--af-success)); }

.diff-chevron { color: var(--af-muted); }

.diff-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.5rem;
  padding: 0.5rem 0.6rem;
  background: hsl(var(--muted-foreground) / 0.02);
  border-top: 1px solid var(--af-border);
}

.diff-side {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.diff-label {
  font-size: 0.65rem;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--af-muted);
  letter-spacing: 0.02em;
}

.diff-content {
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 4px;
  padding: 0.35rem;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--af-fg);
  margin: 0;
}

.diff-content.old { color: var(--af-muted); }

.diff-editor {
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 4px;
  padding: 0.35rem;
  color: var(--af-fg);
  resize: vertical;
  outline: none;
  width: 100%;
  box-sizing: border-box;
}

.diff-editor:focus {
  border-color: hsl(var(--primary) / 0.4);
}

.gate-actions {
  display: flex;
  gap: 0.5rem;
}

.approve-btn,
.reject-btn,
.review-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.9rem;
  border: none;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}

.approve-btn {
  background: linear-gradient(135deg, var(--vp-c-brand-1) 0%, var(--vp-c-brand-2) 100%);
  color: #fff;
}

.reject-btn {
  background: transparent;
  color: var(--af-fg);
  border: 1px solid var(--af-border);
}

.review-btn {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
  border: 1px solid var(--af-border);
}

.approve-btn:hover,
.reject-btn:hover,
.review-btn:hover {
  opacity: 0.85;
}
</style>
