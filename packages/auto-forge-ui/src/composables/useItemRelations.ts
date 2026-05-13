import { ref, computed } from 'vue'
import type { SpecItem } from '@/types/specs'

const API_BASE = '/api/forge/specs'

export interface RelatedItem {
  id: string
  title: string
  section_type: string
  status: string
}

export function useItemRelations(project: string) {
  const loading = ref(false)
  const parents = ref<RelatedItem[]>([])
  const children = ref<RelatedItem[]>([])

  async function loadRelations(itemId: string) {
    loading.value = true
    try {
      const resp = await fetch(
        `${API_BASE}/${encodeURIComponent(project)}/related/${encodeURIComponent(itemId)}`
      )
      if (!resp.ok) throw new Error(`Failed to load relations: ${resp.status}`)
      const data = await resp.json()
      parents.value = data.parents || []
      children.value = data.children || []
    } catch (e) {
      console.error('Failed to load relations:', e)
      parents.value = []
      children.value = []
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    parents,
    children,
    loadRelations,
  }
}
