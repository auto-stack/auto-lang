<template>
  <div class="jades-view">
    <div class="jades-header">
      <h2>Specs</h2>
      <div class="jades-actions">
        <button class="jades-btn" @click="triggerDriftCheck">
          <RefreshCw :size="14" />
          Drift Check
        </button>
        <button class="jades-btn" @click="aiEnrich">
          <Sparkles :size="14" />
          AI Enrich
        </button>
      </div>
    </div>
    <div class="jades-body">
      <div class="section-nav">
        <div
          v-for="section in sections"
          :key="section.id"
          class="section-nav-item"
          :class="{ active: activeSection === section.id, drift: section.status === 'drift' }"
          @click="activeSection = section.id"
        >
          <span class="nav-status" :class="section.status" />
          <span class="nav-label">{{ section.title }}</span>
          <span v-if="section.status === 'drift'" class="nav-drift">!</span>
        </div>
      </div>
      <div class="section-editor">
        <div v-if="currentSection" class="editor-pane">
          <div class="editor-header">
            <h3>{{ currentSection.title }}</h3>
            <div class="editor-badges">
              <span class="badge" :class="currentSection.status">{{ currentSection.status }}</span>
              <span v-if="currentSection.last_verified" class="badge meta">
                Verified: {{ formatDate(currentSection.last_verified) }}
              </span>
            </div>
          </div>
          <textarea
            v-model="currentSection.content"
            class="editor-textarea"
            spellcheck="false"
          />
          <div class="editor-footer">
            <button class="save-btn" @click="saveSection">Save</button>
          </div>
        </div>
        <div v-else-if="isLoading" class="editor-empty">
          <span class="loading">Loading Specs…</span>
        </div>
        <div v-else class="editor-empty">
          <BookOpen :size="32" />
          <p>Select a section from the sidebar</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { RefreshCw, Sparkles, BookOpen } from 'lucide-vue-next'
import { useLedger } from '@/composables/useLedger'
import type { LedgerSection } from '@/types/ledger'

const { document, isLoading, error, loadDocument, saveSection: saveLedgerSection } = useLedger()

const activeSection = ref<string>('goals')
const project = ref('.')

const sections = computed(() => document.value?.sections ?? [])

const currentSection = computed(() =>
  document.value?.sections.find((s) => s.id === activeSection.value) ?? null
)

function formatDate(ts: number): string {
  return new Date(ts).toLocaleDateString()
}

async function triggerDriftCheck() {
  try {
    const resp = await fetch(`/api/smith/ledger/${encodeURIComponent(project.value)}/drift-check`, {
      method: 'POST',
    })
    const data = await resp.json()
    alert(`Drift check: ${data.drift_detected ? 'DRIFT DETECTED' : 'No drift detected'} (${data.sections_checked} sections checked)`)
  } catch {
    alert('Drift check failed.')
  }
}

function aiEnrich() {
  alert('AI Enrich: In the full implementation, the AI would analyze the codebase and propose Ledger updates.')
}

async function saveSection() {
  const section = currentSection.value
  if (!section) return
  await saveLedgerSection(project.value, section)
  if (error.value) {
    alert('Save failed: ' + error.value)
  }
}

onMounted(() => {
  loadDocument(project.value)
})
</script>

<style scoped>
.jades-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.jades-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 1.25rem;
  flex-shrink: 0;
}

.jades-header h2 {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--af-fg);
}

.jades-actions {
  display: flex;
  gap: 0.4rem;
}

.jades-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  background: transparent;
  color: var(--af-muted);
  border: 1px solid var(--af-border);
  border-radius: 6px;
  padding: 0.3rem 0.6rem;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s;
}

.jades-btn:hover {
  background: hsl(var(--muted-foreground) / 0.05);
  color: var(--af-fg);
}

.jades-body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.section-nav {
  width: 220px;
  background: transparent;
  border-right: 1px solid var(--af-border);
  padding: 0.25rem 0.5rem;
  overflow-y: auto;
  flex-shrink: 0;
}

.section-nav-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  font-size: 0.8rem;
  color: var(--af-fg);
}

.section-nav-item:hover {
  background: hsl(var(--muted-foreground) / 0.04);
}

.section-nav-item.active {
  background: hsl(var(--primary) / 0.06);
  color: var(--af-primary);
  font-weight: 500;
}

.section-nav-item.drift {
  color: hsl(var(--af-error));
}

.nav-status {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.nav-status.draft { background: var(--af-muted); }
.nav-status.approved { background: hsl(var(--af-info)); }
.nav-status.in_progress { background: hsl(var(--af-warning)); }
.nav-status.verified { background: hsl(var(--af-success)); }
.nav-status.archived { background: var(--af-border); }
.nav-status.drift { background: hsl(var(--af-error)); }

.nav-label {
  flex: 1;
}

.nav-drift {
  color: hsl(var(--af-error));
  font-size: 0.75rem;
  font-weight: 600;
}

.section-editor {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 1.25rem;
  border-bottom: 1px solid var(--af-border);
  flex-shrink: 0;
}

.editor-header h3 {
  font-size: 0.9rem;
  font-weight: 500;
}

.editor-badges {
  display: flex;
  gap: 0.4rem;
}

.badge {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--af-muted);
}

.badge.draft { color: var(--af-muted); }
.badge.approved { color: hsl(var(--af-info)); }
.badge.in_progress { color: hsl(var(--af-warning)); }
.badge.verified { color: hsl(var(--af-success)); }
.badge.drift { color: hsl(var(--af-error)); }
.badge.meta { color: var(--af-muted); }

.editor-textarea {
  flex: 1;
  background: var(--af-bg);
  border: none;
  padding: 1rem 1.25rem;
  color: var(--af-fg);
  font-size: 0.9rem;
  line-height: 1.6;
  resize: none;
  outline: none;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  white-space: pre;
  overflow-wrap: normal;
  overflow-x: auto;
}

.editor-footer {
  display: flex;
  justify-content: flex-end;
  padding: 0.5rem 1.25rem;
  border-top: 1px solid var(--af-border);
  flex-shrink: 0;
}

.save-btn {
  background: linear-gradient(135deg, var(--vp-c-brand-1) 0%, var(--vp-c-brand-2) 100%);
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 0.35rem 0.9rem;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}

.save-btn:hover {
  opacity: 0.85;
}

.editor-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  color: var(--af-muted);
  flex: 1;
}

.loading {
  font-size: 0.9rem;
  color: var(--af-muted);
}
</style>
