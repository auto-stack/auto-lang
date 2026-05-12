<template>
  <div class="jades-view">
    <div class="jades-body">
      <!-- Sidebar -->
      <div class="section-nav" :class="{ collapsed: sectionNavCollapsed }">
        <div class="section-nav-header">
          <span class="section-nav-title">Specs</span>
          <button
            class="section-nav-collapse-btn"
            @click="sectionNavCollapsed = !sectionNavCollapsed"
            title="Toggle sidebar"
          >
            <PanelLeft :size="14" />
          </button>
        </div>
        <div
          v-for="section in filteredSections"
          :key="section.id"
          class="section-nav-item"
          :class="{ active: activeSection === section.id, drift: section.status === 'drift' }"
          @click="activeSection = section.id"
        >
          <div class="section-top">
            <span class="section-name">{{ section.title }}</span>
            <button
              class="section-edit-btn"
              title="Edit"
              @click.stop="activeSection = section.id"
            >
              <Pencil :size="11" />
            </button>
          </div>
          <div class="section-meta">
            <span class="section-count">{{ lineCount(section.content) }} items</span>
            <span class="section-status" :class="section.status">{{ section.status }}</span>
          </div>
        </div>
      </div>
      <!-- Content pane -->
      <div class="section-editor">
        <div class="content-header">
          <div class="header-title-row">
            <button v-if="sectionNavCollapsed" class="section-nav-toggle-btn" @click="sectionNavCollapsed = false" title="Show sections">
              <PanelLeft :size="16" />
            </button>
          </div>
          <div class="header-center">
            <div v-if="projectName" class="header-project">{{ projectName }}</div>
            <div class="header-search">
              <Search :size="13" />
              <input
                v-model="specSearch"
                type="text"
                class="search-input"
                placeholder="Search sections..."
              />
            </div>
          </div>
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
        <div class="editor-scroll">
          <div v-if="currentSection" class="editor-pane">
            <div class="editor-header">
              <h3>{{ currentSection.title }}</h3>
              <div class="editor-header-right">
                <div class="editor-badges">
                  <span class="badge" :class="currentSection.status">{{ currentSection.status }}</span>
                  <span v-if="currentSection.last_verified" class="badge meta">
                    Verified: {{ formatDate(currentSection.last_verified) }}
                  </span>
                </div>
                <button
                  class="mode-toggle-btn"
                  :title="editMode ? 'Preview' : 'Edit'"
                  @click="editMode = !editMode"
                >
                  <Eye v-if="editMode" :size="14" />
                  <FileEdit v-else :size="14" />
                  <span>{{ editMode ? 'Preview' : 'Edit' }}</span>
                </button>
              </div>
            </div>
            <div v-if="!editMode" class="editor-viewer">
              <MarkdownRender :content="currentSection.content" :final="true" />
            </div>
            <textarea
              v-else
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { RefreshCw, Sparkles, BookOpen, Search, Pencil, PanelLeft, Eye, FileEdit } from 'lucide-vue-next'
import { MarkdownRender } from 'markstream-vue'
import { useLedger } from '@/composables/useLedger'
import type { LedgerSection } from '@/types/ledger'

const { document, isLoading, error, loadDocument, saveSection: saveLedgerSection } = useLedger()

const SPECS_SIDEBAR_KEY = 'autoforge-specs-sidebar-collapsed'

const activeSection = ref<string>('goals')
const project = ref('.')
const specSearch = ref('')
const sectionNavCollapsed = ref(localStorage.getItem(SPECS_SIDEBAR_KEY) === 'true')
const editMode = ref(false)

watch(sectionNavCollapsed, (v) => {
  localStorage.setItem(SPECS_SIDEBAR_KEY, String(v))
})

const projectName = computed(() => {
  const p = project.value
  if (!p || p === '.') return null
  const parts = p.replace(/\\/g, '/').split('/').filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : null
})

const DEFAULT_SECTIONS: LedgerSection[] = [
  { id: 'goals', type: 'goals', title: '🎯 Goals', content: '', status: 'draft', last_modified: Date.now() },
  { id: 'architecture', type: 'architecture', title: '🏗️ Architecture', content: '', status: 'draft', last_modified: Date.now() },
  { id: 'designs', type: 'designs', title: '🎨 Designs', content: '', status: 'draft', last_modified: Date.now() },
  { id: 'plans', type: 'plans', title: '📅 Plans', content: '', status: 'draft', last_modified: Date.now() },
  { id: 'reviews', type: 'reviews', title: '📝 Reviews', content: '', status: 'draft', last_modified: Date.now() },
  { id: 'reports', type: 'reports', title: '📊 Reports', content: '', status: 'draft', last_modified: Date.now() },
  { id: 'apis', type: 'apis', title: '🔌 APIs', content: '', status: 'draft', last_modified: Date.now() },
]

const sections = computed(() => {
  const loaded = document.value?.sections
  if (loaded && loaded.length > 0) return loaded
  return DEFAULT_SECTIONS
})

const filteredSections = computed(() => {
  const q = specSearch.value.trim().toLowerCase()
  if (!q) return sections.value
  return sections.value.filter((s) =>
    s.title.toLowerCase().includes(q) ||
    s.id.toLowerCase().includes(q) ||
    s.content.toLowerCase().includes(q)
  )
})

const currentSection = computed(() =>
  document.value?.sections.find((s) => s.id === activeSection.value) ?? null
)

function formatDate(ts: number): string {
  return new Date(ts).toLocaleDateString()
}

function lineCount(content: string): number {
  if (!content.trim()) return 0
  return content.trim().split('\n').filter((l) => l.trim()).length
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

.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 1.25rem;
  flex-shrink: 0;
  border-bottom: 1px solid var(--af-border);
}

.header-center {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.15rem;
  flex: 1;
  max-width: 520px;
  margin: 0 auto;
}

.header-project {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--af-primary);
  line-height: 1;
}

.header-search {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  width: 100%;
  padding: 0.35rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.06);
  border: 1px solid hsl(var(--muted-foreground) / 0.12);
  border-radius: 6px;
  color: var(--af-muted);
  transition: border-color 0.15s, background 0.15s;
}

.header-search:focus-within {
  border-color: hsl(var(--primary) / 0.35);
  background: hsl(var(--muted-foreground) / 0.04);
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--af-fg);
  font-size: 0.85rem;
  font-family: inherit;
  min-width: 0;
}

.search-input::placeholder {
  color: var(--af-muted);
  font-size: 0.8rem;
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
  padding: 0 0.5rem;
  overflow-y: auto;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  transition: width 0.2s ease, margin-left 0.2s ease;
}

.section-nav.collapsed {
  width: 0;
  margin-left: -1px;
  overflow: hidden;
  padding: 0;
}

.section-nav-header {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.75rem 1rem;
  flex-shrink: 0;
  height: 48px;
}

.section-nav-title {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--af-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex: 1;
  line-height: 1;
}

.section-nav-collapse-btn,
.section-nav-toggle-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  background: transparent;
  border: none;
  border-radius: 5px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.section-nav-collapse-btn:hover,
.section-nav-toggle-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
  width: 80px;
}

.section-nav-item {
  padding: 0.5rem 0.6rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.section-nav-item:hover {
  background: hsl(var(--muted-foreground) / 0.05);
}

.section-nav-item.active {
  background: hsl(var(--primary) / 0.06);
  border-left: 2px solid var(--af-primary);
  margin-left: -2px;
}

.section-nav-item.drift .section-name {
  color: hsl(var(--af-error));
}

.section-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-name {
  font-size: 0.8rem;
  color: var(--af-fg);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.section-edit-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--af-muted);
  cursor: pointer;
  opacity: 0;
  transition: all 0.15s;
}

.section-nav-item:hover .section-edit-btn {
  opacity: 1;
}

.section-edit-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.section-meta {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-top: 0.2rem;
}

.section-count {
  font-size: 0.65rem;
  color: var(--af-muted);
}

.section-status {
  font-size: 0.6rem;
  font-weight: 500;
  color: var(--af-muted);
}

.section-status.draft { color: var(--af-muted); }
.section-status.approved { color: hsl(var(--af-info)); }
.section-status.in_progress { color: hsl(var(--af-warning)); }
.section-status.verified { color: hsl(var(--af-success)); }
.section-status.archived { color: var(--af-border); }
.section-status.drift { color: hsl(var(--af-error)); }

.section-editor {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.editor-pane {
  display: flex;
  flex-direction: column;
  min-height: 100%;
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

.editor-header-right {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.editor-badges {
  display: flex;
  gap: 0.4rem;
}

.mode-toggle-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  background: transparent;
  border: 1px solid var(--af-border);
  border-radius: 6px;
  padding: 0.25rem 0.6rem;
  font-size: 0.75rem;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.mode-toggle-btn:hover {
  background: hsl(var(--muted-foreground) / 0.06);
  color: var(--af-fg);
  border-color: hsl(var(--primary) / 0.3);
}

.editor-viewer {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 1rem 1.25rem;
  background: var(--af-bg);
}

.editor-viewer :deep(.markstream-vue) {
  --ms-text-body: 0.9rem;
  max-width: 720px;
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
