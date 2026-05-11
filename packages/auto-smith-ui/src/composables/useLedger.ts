import { ref } from 'vue'
import type { LedgerDocument, LedgerSection } from '@/types/ledger'

const API_BASE = '/api/smith'

// ─── Singleton state ────────────────────────────────────────────────────────
const _document = ref<LedgerDocument | null>(null)
const _isLoading = ref(false)
const _error = ref<string | null>(null)

export function useLedger() {
  const document = _document
  const isLoading = _isLoading
  const error = _error

  async function loadDocument(project: string = '.') {
    isLoading.value = true
    error.value = null
    try {
      const resp = await fetch(`${API_BASE}/ledger/${encodeURIComponent(project)}`)
      if (!resp.ok) throw new Error(`Failed to load ledger: ${resp.status}`)
      const data: LedgerDocument = await resp.json()
      document.value = data
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function saveSection(project: string, section: LedgerSection) {
    try {
      const resp = await fetch(
        `${API_BASE}/ledger/${encodeURIComponent(project)}/${encodeURIComponent(section.id)}`,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content: section.content, status: section.status }),
        }
      )
      if (!resp.ok) throw new Error(`Failed to save section: ${resp.status}`)
      // Reload to get updated version
      await loadDocument(project)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function saveDocument(project: string, doc: LedgerDocument) {
    try {
      const resp = await fetch(`${API_BASE}/ledger/${encodeURIComponent(project)}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(doc),
      })
      if (!resp.ok) throw new Error(`Failed to save ledger: ${resp.status}`)
      const data: LedgerDocument = await resp.json()
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
  }
}
