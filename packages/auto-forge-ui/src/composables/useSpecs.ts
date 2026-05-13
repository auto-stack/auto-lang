import { ref } from 'vue'
import type { SpecsDocument, SpecsSection, SpecItem } from '@/types/specs'

const API_BASE = '/api/forge/specs'

// ─── Singleton state ────────────────────────────────────────────────────────
const _document = ref<SpecsDocument | null>(null)
const _isLoading = ref(false)
const _error = ref<string | null>(null)

export function useSpecs() {
  const document = _document
  const isLoading = _isLoading
  const error = _error

  async function loadDocument(project: string = 'auto-lang') {
    isLoading.value = true
    error.value = null
    try {
      const resp = await fetch(`${API_BASE}/${encodeURIComponent(project)}`)
      if (!resp.ok) throw new Error(`Failed to load specs: ${resp.status}`)
      const data: SpecsDocument = await resp.json()
      document.value = data
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function saveSection(project: string, section: SpecsSection) {
    try {
      const resp = await fetch(
        `${API_BASE}/${encodeURIComponent(project)}/${encodeURIComponent(section.id)}`,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content: section.content, status: section.status }),
        }
      )
      if (!resp.ok) throw new Error(`Failed to save section: ${resp.status}`)
      await loadDocument(project)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function saveDocument(project: string, doc: SpecsDocument) {
    try {
      const resp = await fetch(`${API_BASE}/${encodeURIComponent(project)}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(doc),
      })
      if (!resp.ok) throw new Error(`Failed to save specs: ${resp.status}`)
      const data: SpecsDocument = await resp.json()
      document.value = data
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  // ─── Helpers ──────────────────────────────────────────────────────────────

  function findItemById(id: string): { item: SpecItem; section: SpecsSection } | null {
    const doc = document.value
    if (!doc) return null
    for (const section of doc.sections) {
      const item = section.items.find((i) => i.id === id)
      if (item) return { item, section }
    }
    return null
  }

  function findSectionByItemId(id: string): SpecsSection | null {
    const doc = document.value
    if (!doc) return null
    return doc.sections.find((s) => s.items.some((i) => i.id === id)) ?? null
  }

  async function rebuildRelations(project: string) {
    try {
      const resp = await fetch(`${API_BASE}/${encodeURIComponent(project)}/rebuild-relations`, {
        method: 'POST',
      })
      if (!resp.ok) throw new Error(`Failed to rebuild relations: ${resp.status}`)
      const data: SpecsDocument = await resp.json()
      document.value = data
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  return {
    document,
    isLoading,
    error,
    loadDocument,
    saveSection,
    saveDocument,
    findItemById,
    findSectionByItemId,
    rebuildRelations,
  }
}
