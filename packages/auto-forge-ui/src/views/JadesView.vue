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
          :class="{ active: activeSection === section.id }"
          @click="activeSection = section.id"
        >
          <div class="section-top">
            <span class="section-name">{{ section.title }}</span>
          </div>
          <div class="section-meta">
            <span class="section-count">{{ section.items?.length || 0 }} items</span>
            <StatusBadge :status="section.status" size="sm" />
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
            <button class="jades-btn" @click="rebuildRelations">
              <Link2 :size="14" />
              Rebuild Links
            </button>
          </div>
        </div>

        <div class="editor-scroll">
          <div v-if="currentSection" class="editor-pane">
            <div class="editor-header">
              <h3>{{ currentSection.title }}</h3>
              <div class="editor-header-right">
                <div class="editor-badges">
                  <StatusBadge :status="currentSection.status" />
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

            <!-- Add item button -->
            <div v-if="!editMode" class="section-toolbar">
              <button class="add-btn" @click="addItem">
                <Plus :size="14" />
                Add {{ sectionTypeLabel }}
              </button>
            </div>

            <!-- Category-specific renderer -->
            <template v-if="!editMode">
              <component
                :is="categoryComponent"
                :items="currentSection.items"
                :project="project"
                :expanded-id="activeItemId"
                @toggle="toggleItem"
                @jump="jumpToItem"
                @edit="startEditItem"
              />
            </template>

            <!-- Edit mode: structured editor or markdown -->
            <template v-else>
              <div v-if="currentSection.items?.length > 0" class="edit-items-list">
                <div
                  v-for="item in currentSection.items"
                  :key="item.id"
                  class="edit-item-row"
                >
                  <div class="edit-item-fields">
                    <label>ID</label>
                    <input v-model="item.id" class="edit-input monospace" disabled />
                    <label>Title</label>
                    <input v-model="item.title" class="edit-input" />
                    <label>Status</label>
                    <select v-model="item.status" class="edit-select">
                      <option v-for="s in allowedStatuses" :key="s" :value="s">{{ s }}</option>
                    </select>
                    <label>Content</label>
                    <textarea v-model="item.content" class="edit-textarea" rows="6" />
                  </div>
                </div>
              </div>
              <div v-else class="editor-viewer">
                <MarkdownRender :content="currentSection.content" :final="true" />
              </div>
              <div class="editor-footer">
                <button class="save-btn" @click="saveSection">Save</button>
                <button class="cancel-btn" @click="editMode = false">Cancel</button>
              </div>
            </template>
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
import {
  RefreshCw, Search, PanelLeft, Eye, FileEdit, BookOpen, Plus, Link2
} from 'lucide-vue-next'
import { MarkdownRender } from 'markstream-vue'
import { useSpecs } from '@/composables/useSpecs'
import type { SpecsSection, SpecItem, SectionType, Status } from '@/types/specs'
import StatusBadge from '@/components/StatusBadge.vue'

// Category components
import GoalsTable from '@/components/category/GoalsTable.vue'
import RequirementsCards from '@/components/category/RequirementsCards.vue'
import TestsCards from '@/components/category/TestsCards.vue'
import GenericItems from '@/components/category/GenericItems.vue'

const { document, isLoading, error, loadDocument, saveDocument, findItemById, findSectionByItemId, rebuildRelations: apiRebuildRelations } = useSpecs()

const SPECS_SIDEBAR_KEY = 'autoforge-specs-sidebar-collapsed'

const activeSection = ref<string>('goals')
const activeItemId = ref<string | null>(null)
const project = ref('auto-lang')
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

const DEFAULT_SECTIONS: SpecsSection[] = [
  { id: 'goals', section_type: 'goals', title: '🎯 Goals', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'requirements', section_type: 'requirements', title: '📐 Requirements', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'architecture', section_type: 'architecture', title: '🏗️ Architecture', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'designs', section_type: 'designs', title: '🎨 Designs', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'plans', section_type: 'plans', title: '📅 Plans', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'todos', section_type: 'todos', title: '☑️ Todos', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'tests', section_type: 'tests', title: '🧪 Tests', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'reviews', section_type: 'reviews', title: '📝 Reviews', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'reports', section_type: 'reports', title: '📊 Reports', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'apis', section_type: 'apis', title: '🔌 APIs', items: [], content: '', status: 'empty', last_modified: Date.now() },
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

const categoryComponent = computed(() => {
  const type = currentSection.value?.section_type
  switch (type) {
    case 'goals': return GoalsTable
    case 'requirements': return RequirementsCards
    case 'tests': return TestsCards
    default: return GenericItems
  }
})

const sectionTypeLabel = computed(() => {
  const type = currentSection.value?.section_type
  if (!type) return 'Item'
  return type.charAt(0).toUpperCase() + type.slice(1).replace('_', ' ')
})

const allowedStatuses = computed((): Status[] => {
  // Simple list of all statuses for the dropdown
  return [
    'empty', 'proposed', 'draft', 'under_review', 'approved',
    'in_progress', 'in_implementation', 'implemented', 'verified',
    'done', 'archived', 'rejected', 'backlog', 'ready', 'in_review',
    'blocked', 'superseded', 'outdated', 'stable', 'deprecated',
    'published', 'analysed', 'obsolete'
  ]
})

function toggleItem(id: string) {
  activeItemId.value = activeItemId.value === id ? null : id
}

function jumpToItem(id: string) {
  const result = findItemById(id)
  if (!result) return
  const section = findSectionByItemId(id)
  if (section) {
    activeSection.value = section.id
    activeItemId.value = id
    // Scroll to item after next tick
    setTimeout(() => {
      const el = document?.value ? null : null // placeholder
      // In a real implementation, we'd querySelector and scrollIntoView
    }, 50)
  }
}

function startEditItem(item: SpecItem) {
  activeItemId.value = item.id
  editMode.value = true
}

function addItem() {
  if (!currentSection.value) return
  const section = currentSection.value
  const prefix = section.id.charAt(0).toUpperCase()
  const seq = section.items.length + 1
  const newItem: SpecItem = {
    id: `${prefix}${seq}`,
    title: `New ${sectionTypeLabel.value}`,
    content: '',
    status: 'draft',
    created_at: Date.now(),
    modified_at: Date.now(),
  }
  section.items.push(newItem)
  editMode.value = true
  activeItemId.value = newItem.id
}

async function saveSection() {
  const section = currentSection.value
  if (!section) return
  // Update section-level content from items for compatibility
  // Serialize items into markdown content
  section.content = serializeItemsToMarkdown(section)
  section.last_modified = Date.now()
  const doc = document.value
  if (doc) {
    await saveDocument(project.value, doc)
    editMode.value = false
  }
  if (error.value) {
    alert('Save failed: ' + error.value)
  }
}

function serializeItemsToMarkdown(section: SpecsSection): string {
  // Simple serialization — category-specific serialization would be better
  const lines: string[] = [`## ${section.title.replace(/^[^\w]+\s*/, '')}`]
  for (const item of section.items) {
    lines.push(`### ${item.id} ${item.title}`)
    lines.push(`**Status:** ${item.status}`)
    if (item.priority) lines.push(`**Priority:** ${item.priority}`)
    if (item.assignee) lines.push(`**Assignee:** ${item.assignee}`)
    if (item.test_file) lines.push(`**Test File:** ${item.test_file}`)
    if (item.depends_on?.length) lines.push(`**Depends on:** ${item.depends_on.join(', ')}`)
    lines.push('')
    lines.push(item.content)
    lines.push('')
  }
  return lines.join('\n')
}

async function triggerDriftCheck() {
  try {
    const resp = await fetch(`/api/forge/specs/${encodeURIComponent(project.value)}/drift-check`, {
      method: 'POST',
    })
    const data = await resp.json()
    alert(`Drift check: ${data.drift_detected ? 'DRIFT DETECTED' : 'No drift detected'} (${data.sections_checked} sections checked)`)
  } catch {
    alert('Drift check failed.')
  }
}

async function rebuildRelations() {
  await apiRebuildRelations(project.value)
  if (error.value) {
    alert('Rebuild failed: ' + error.value)
  } else {
    alert('Relations rebuilt successfully.')
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

.jades-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ─── Sidebar ─────────────────────────────────────────────── */

.section-nav {
  width: 220px;
  min-width: 220px;
  border-right: 1px solid var(--af-border);
  background: hsl(var(--muted-foreground) / 0.02);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  transition: width 0.2s ease, min-width 0.2s ease;
}

.section-nav.collapsed {
  width: 0;
  min-width: 0;
  padding: 0;
  border-right: none;
  overflow: hidden;
}

.section-nav-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--af-border);
}

.section-nav-title {
  font-size: 0.85rem;
  font-weight: 700;
  color: var(--af-fg);
}

.section-nav-collapse-btn {
  background: none;
  border: none;
  color: var(--af-muted);
  cursor: pointer;
  padding: 0.2rem;
  border-radius: 4px;
}

.section-nav-collapse-btn:hover {
  color: var(--af-fg);
  background: hsl(var(--muted-foreground) / 0.08);
}

.section-nav-item {
  padding: 0.6rem 1rem;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background 0.12s;
}

.section-nav-item:hover {
  background: hsl(var(--muted-foreground) / 0.05);
}

.section-nav-item.active {
  background: hsl(var(--primary) / 0.06);
  border-left-color: hsl(var(--primary));
}

.section-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.2rem;
}

.section-name {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--af-fg);
}

.section-meta {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.section-count {
  font-size: 0.7rem;
  color: var(--af-muted);
}

/* ─── Content Pane ────────────────────────────────────────── */

.section-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 1rem;
  border-bottom: 1px solid var(--af-border);
  gap: 1rem;
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.section-nav-toggle-btn {
  background: none;
  border: none;
  color: var(--af-muted);
  cursor: pointer;
  padding: 0.3rem;
  border-radius: 4px;
}

.section-nav-toggle-btn:hover {
  color: var(--af-fg);
  background: hsl(var(--muted-foreground) / 0.08);
}

.header-center {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex: 1;
  justify-content: center;
}

.header-project {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--af-fg);
  white-space: nowrap;
}

.header-search {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: hsl(var(--muted-foreground) / 0.06);
  border-radius: 8px;
  padding: 0.35rem 0.6rem;
  min-width: 200px;
  max-width: 320px;
  flex: 1;
}

.header-search svg {
  color: var(--af-muted);
  flex-shrink: 0;
}

.search-input {
  background: transparent;
  border: none;
  outline: none;
  font-size: 0.8rem;
  color: var(--af-fg);
  width: 100%;
}

.search-input::placeholder {
  color: var(--af-muted);
}

.jades-actions {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.jades-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.7rem;
  font-size: 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--muted-foreground) / 0.04);
  color: var(--af-fg);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.jades-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  border-color: hsl(var(--primary) / 0.3);
}

.editor-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.editor-pane {
  max-width: 960px;
  margin: 0 auto;
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--af-border);
}

.editor-header h3 {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--af-fg);
  margin: 0;
}

.editor-header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.editor-badges {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.mode-toggle-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.35rem 0.6rem;
  font-size: 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--muted-foreground) / 0.04);
  color: var(--af-fg);
  cursor: pointer;
}

.mode-toggle-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
}

.section-toolbar {
  margin-bottom: 0.75rem;
}

.add-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.4rem 0.7rem;
  font-size: 0.8rem;
  border-radius: 6px;
  border: 1px dashed var(--af-border);
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
}

.add-btn:hover {
  color: hsl(var(--primary));
  border-color: hsl(var(--primary) / 0.4);
  background: hsl(var(--primary) / 0.04);
}

.editor-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--af-muted);
  gap: 0.5rem;
}

.loading {
  font-size: 0.9rem;
  color: var(--af-muted);
}

/* ─── Edit Mode ───────────────────────────────────────────── */

.edit-items-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.edit-item-row {
  padding: 1rem;
  border-radius: 8px;
  background: hsl(var(--muted-foreground) / 0.03);
  border: 1px solid var(--af-border);
}

.edit-item-fields {
  display: grid;
  grid-template-columns: 80px 1fr;
  gap: 0.5rem;
  align-items: center;
}

.edit-item-fields label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--af-muted);
  text-align: right;
}

.edit-input,
.edit-select,
.edit-textarea {
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--background));
  color: var(--af-fg);
  font-size: 0.85rem;
  outline: none;
}

.edit-input:focus,
.edit-select:focus,
.edit-textarea:focus {
  border-color: hsl(var(--primary) / 0.5);
}

.edit-input.monospace {
  font-family: monospace;
}

.edit-textarea {
  resize: vertical;
  min-height: 120px;
  grid-column: 2;
}

.editor-footer {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--af-border);
}

.save-btn {
  padding: 0.45rem 1rem;
  font-size: 0.8rem;
  border-radius: 6px;
  border: none;
  background: hsl(var(--primary));
  color: white;
  cursor: pointer;
  font-weight: 600;
}

.save-btn:hover {
  opacity: 0.9;
}

.cancel-btn {
  padding: 0.45rem 1rem;
  font-size: 0.8rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
}

.cancel-btn:hover {
  color: var(--af-fg);
  background: hsl(var(--muted-foreground) / 0.05);
}
</style>
