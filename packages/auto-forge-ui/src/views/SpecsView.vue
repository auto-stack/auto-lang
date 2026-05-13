<template>
  <div class="specs-view">
    <div class="specs-body">
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
          <div class="specs-actions">
            <button class="specs-btn" @click="triggerDriftCheck">
              <RefreshCw :size="14" />
              Drift Check
            </button>
            <button class="specs-btn" @click="rebuildRelations">
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
              </div>
            </div>

            <!-- Add item button -->
            <div class="section-toolbar">
              <button class="add-btn" @click="addItem">
                <Plus :size="14" />
                Add {{ sectionTypeLabel }}
              </button>
            </div>

            <!-- Category-specific renderer -->
            <component
              :is="categoryComponent"
              :items="currentSection.items"
              :project="project"
              :expanded-id="activeItemId"
              :editing-id="editingItemId"
              @toggle="toggleItem"
              @jump="jumpToItem"
              @edit="startEditItem"
              @status-change="handleStatusChange"
              @delete="handleDelete"
              @save="handleSave"
              @cancel-edit="cancelEdit"
            />
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
  RefreshCw, Search, PanelLeft, BookOpen, Plus, Link2
} from 'lucide-vue-next'
import { useSpecs } from '@/composables/useSpecs'
import type { SpecsSection, SpecItem, SectionType, Status } from '@/types/specs'
import StatusBadge from '@/components/StatusBadge.vue'

// Category components
import GoalsTable from '@/components/category/GoalsTable.vue'
import ArchitectureCards from '@/components/category/ArchitectureCards.vue'
import DesignCards from '@/components/category/DesignCards.vue'
import PlanCards from '@/components/category/PlanCards.vue'
import TestsCards from '@/components/category/TestsCards.vue'
import ReviewCards from '@/components/category/ReviewCards.vue'
import ReportCards from '@/components/category/ReportCards.vue'
import ApiCards from '@/components/category/ApiCards.vue'

const { document, isLoading, error, loadDocument, saveDocument, findItemById, findSectionByItemId, rebuildRelations: apiRebuildRelations } = useSpecs()

const SPECS_SIDEBAR_KEY = 'autoforge-specs-sidebar-collapsed'

const activeSection = ref<string>('goals')
const activeItemId = ref<string | null>(null)
const editingItemId = ref<string | null>(null)
const project = ref('auto-lang')
const specSearch = ref('')
const sectionNavCollapsed = ref(localStorage.getItem(SPECS_SIDEBAR_KEY) === 'true')
const flashItemId = ref<string | null>(null)

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
  { id: 'architecture', section_type: 'architecture', title: '🏗️ Architecture', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'designs', section_type: 'designs', title: '🎨 Designs', items: [], content: '', status: 'empty', last_modified: Date.now() },
  { id: 'plans', section_type: 'plans', title: '📅 Plans', items: [], content: '', status: 'empty', last_modified: Date.now() },
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
    case 'architecture': return ArchitectureCards
    case 'designs': return DesignCards
    case 'plans': return PlanCards
    case 'tests': return TestsCards
    case 'reviews': return ReviewCards
    case 'reports': return ReportCards
    case 'apis': return ApiCards
    default: return null
  }
})

const sectionTypeLabel = computed(() => {
  const type = currentSection.value?.section_type
  if (!type) return 'Item'
  return type.charAt(0).toUpperCase() + type.slice(1).replace('_', ' ')
})

function handleStatusChange(payload: { id: string; status: Status }) {
  const section = currentSection.value
  if (!section) return
  const item = section.items.find((i) => i.id === payload.id)
  if (item) {
    item.status = payload.status
    item.modified_at = Date.now()
    saveSection()
  }
}

function handleSave(updated: SpecItem) {
  const section = currentSection.value
  if (!section) return
  const idx = section.items.findIndex((i) => i.id === updated.id)
  if (idx >= 0) {
    section.items[idx] = updated
    saveSection()
  }
}

function handleDelete(itemId: string) {
  const section = currentSection.value
  if (!section) return
  const idx = section.items.findIndex((i) => i.id === itemId)
  if (idx >= 0) {
    section.items.splice(idx, 1)
    if (activeItemId.value === itemId) activeItemId.value = null
    saveSection()
  }
}

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
    flashItemId.value = id
    setTimeout(() => { flashItemId.value = null }, 2000)
  }
}

function startEditItem(item: SpecItem) {
  activeItemId.value = item.id
  editingItemId.value = item.id
}

function cancelEdit() {
  editingItemId.value = null
}

function addItem() {
  if (!currentSection.value) return
  const section = currentSection.value
  const prefixMap: Record<string, string> = {
    goals: 'G', architecture: 'A', designs: 'D', plans: 'P',
    tests: 'S', reviews: 'V', reports: 'X', apis: 'I'
  }
  const prefix = prefixMap[section.section_type] || section.id.charAt(0).toUpperCase()
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
  activeItemId.value = newItem.id
  saveSection()
}

async function saveSection() {
  const section = currentSection.value
  if (!section) return
  section.content = serializeItemsToMarkdown(section)
  section.last_modified = Date.now()
  const doc = document.value
  if (doc) {
    await saveDocument(project.value, doc)
  }
  if (error.value) {
    alert('Save failed: ' + error.value)
  }
}

function serializeItemsToMarkdown(section: SpecsSection): string {
  const lines: string[] = [`## ${section.title.replace(/^[^\w]+\s*/, '')}`]
  for (const item of section.items) {
    lines.push(`### ${item.id} ${item.title}`)
    lines.push(`**Status:** ${item.status}`)
    if (item.priority) lines.push(`**Priority:** ${item.priority}`)
    if (item.assignee) lines.push(`**Assignee:** ${item.assignee}`)
    if (item.test_file) lines.push(`**Test File:** ${item.test_file}`)
    if (item.file) lines.push(`**File:** ${item.file}`)
    if (item.milestone) lines.push(`**Milestone:** ${item.milestone}`)
    if (item.module) lines.push(`**Module:** ${item.module}`)
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
.specs-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.specs-body {
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

.specs-actions {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.specs-btn {
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

.specs-btn:hover {
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

/* Flash animation for jump-to-item */
@keyframes flash-highlight {
  0% { background: hsl(48 100% 60% / 0.35); }
  100% { background: transparent; }
}

:deep(.spec-item-row.flash) {
  animation: flash-highlight 1.5s ease-out;
}

</style>
