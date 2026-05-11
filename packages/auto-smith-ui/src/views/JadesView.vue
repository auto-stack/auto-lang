<template>
  <div class="jades-view">
    <div class="jades-header">
      <h2>The Jades · 玉简</h2>
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
        <div v-else class="editor-empty">
          <BookOpen :size="32" />
          <p>Select a section from the sidebar</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { RefreshCw, Sparkles, BookOpen } from 'lucide-vue-next'
import type { LedgerSection } from '@/types/ledger'

const sections = ref<LedgerSection[]>([
  {
    id: 'goals',
    type: 'goals',
    title: '📋 Goals',
    status: 'in_progress',
    content: '- Implement user authentication system\n- Add JWT token flow\n- Support OAuth2 providers',
    last_modified: Date.now(),
  },
  {
    id: 'requirements',
    type: 'requirements',
    title: '📐 Requirements',
    status: 'drift',
    content: 'R1.1: Sessions must support create → active → idle → closed states.\nR1.2: Max idle time: 5 minutes.\nR2.1: Use JWT instead of session cookies.',
    last_modified: Date.now() - 86400000,
    last_verified: Date.now() - 172800000,
  },
  {
    id: 'analysis',
    type: 'analysis',
    title: '🔍 Analysis',
    status: 'draft',
    content: 'The AutovmReplSession holds VM state across cell executions.\nStack overflow risk on deep expressions in test mode.\nMitigation: limit recursion depth in tests.',
    last_modified: Date.now(),
  },
  {
    id: 'plans',
    type: 'plans',
    title: '📅 Plans',
    status: 'approved',
    content: 'Phase 5: Quality + AI Experience\n- 5.1: Test coverage\n- 5.2: AI streaming (SSE)\n- 5.3: One-click code extraction',
    last_modified: Date.now(),
  },
  {
    id: 'todos',
    type: 'todos',
    title: '✅ Todos',
    status: 'in_progress',
    content: '- [x] Add backend unit tests (11 passing)\n- [x] Add frontend Vitest (14 passing)\n- [ ] Add e2e tests for AI streaming',
    last_modified: Date.now(),
  },
  {
    id: 'reports',
    type: 'reports',
    title: '📊 Reports',
    status: 'draft',
    content: 'Coverage Report (2026-05-11):\nnotebook/mod.rs: 87%\nnotebook/ai.rs: 62%\nroutes/notebook.rs: 45%',
    last_modified: Date.now(),
  },
  {
    id: 'reviews',
    type: 'reviews',
    title: '📝 Reviews',
    status: 'draft',
    content: 'REV-1: Security review required\nThe dirty-cell re-execution queue may skip edge cases where upstream cells have side effects.',
    last_modified: Date.now(),
  },
])

const activeSection = ref<string>('goals')

const currentSection = computed(() =>
  sections.value.find((s) => s.id === activeSection.value)
)

function formatDate(ts: number): string {
  return new Date(ts).toLocaleDateString()
}

function triggerDriftCheck() {
  alert('Drift check: In the full implementation, this would compare code against specs and flag divergences.')
}

function aiEnrich() {
  alert('AI Enrich: In the full implementation, the AI would analyze the codebase and propose Ledger updates.')
}

function saveSection() {
  if (currentSection.value) {
    currentSection.value.last_modified = Date.now()
    alert('Section saved (mock).')
  }
}
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
  padding: 0.75rem 1rem;
  background: var(--af-card);
  border-bottom: 1px solid var(--af-border);
  flex-shrink: 0;
}

.jades-header h2 {
  font-size: 1rem;
  font-weight: 600;
  color: hsl(var(--af-jades));
}

.jades-actions {
  display: flex;
  gap: 0.5rem;
}

.jades-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  background: var(--af-secondary);
  color: var(--af-fg);
  border: 1px solid var(--af-border);
  border-radius: 6px;
  padding: 0.35rem 0.65rem;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s;
}

.jades-btn:hover {
  background: var(--af-input);
}

.jades-body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.section-nav {
  width: 240px;
  background: var(--af-card);
  border-right: 1px solid var(--af-border);
  padding: 0.5rem;
  overflow-y: auto;
  flex-shrink: 0;
}

.section-nav-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  font-size: 0.85rem;
  color: var(--af-fg);
}

.section-nav-item:hover {
  background: var(--af-secondary);
}

.section-nav-item.active {
  background: hsl(var(--af-jades) / 0.12);
  color: hsl(var(--af-jades));
}

.section-nav-item.drift {
  color: hsl(var(--af-error));
}

.nav-status {
  width: 8px;
  height: 8px;
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
  background: hsl(var(--af-error));
  color: #fff;
  font-size: 0.65rem;
  font-weight: 700;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
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
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--af-border);
  flex-shrink: 0;
}

.editor-header h3 {
  font-size: 0.95rem;
  font-weight: 600;
}

.editor-badges {
  display: flex;
  gap: 0.4rem;
}

.badge {
  font-size: 0.65rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
}

.badge.draft { background: hsl(var(--af-muted) / 0.15); color: var(--af-muted); }
.badge.approved { background: hsl(var(--af-info) / 0.15); color: hsl(var(--af-info)); }
.badge.in_progress { background: hsl(var(--af-warning) / 0.15); color: hsl(var(--af-warning)); }
.badge.verified { background: hsl(var(--af-success) / 0.15); color: hsl(var(--af-success)); }
.badge.drift { background: hsl(var(--af-error) / 0.15); color: hsl(var(--af-error)); }
.badge.meta { background: var(--af-secondary); color: var(--af-muted); }

.editor-textarea {
  flex: 1;
  background: var(--af-bg);
  border: none;
  padding: 1rem;
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
  padding: 0.75rem 1rem;
  border-top: 1px solid var(--af-border);
  flex-shrink: 0;
}

.save-btn {
  background: hsl(var(--af-jades));
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 0.4rem 1rem;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
}

.save-btn:hover {
  opacity: 0.9;
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
</style>
