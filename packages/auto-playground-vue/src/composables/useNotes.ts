import { ref, computed, shallowRef } from 'vue'
import type { NoteMeta } from '../types'

/** notes.json schema v1（scripts/build-playground-notes.mjs 生成，Playground 设计 §5.2）。 */
export interface NoteGroup {
  id: string
  title: string
  order: number
  source: string
  notes: NoteMeta[]
}

export interface NotesManifest {
  version: number
  groups: NoteGroup[]
}

/** 笔记 + 所属分组（flatNotes / 搜索结果元素，分组归属保留）。 */
export interface NoteWithGroup {
  note: NoteMeta
  group: NoteGroup
}

export interface UseNotesOptions {
  /** manifest URL；默认 /playground-data/notes.json（SPA 宿主经参数改 base）。 */
  base?: string
  /** 挂载即加载（默认 true）。 */
  immediate?: boolean
}

/**
 * Notes manifest 加载与索引（Plan 582，Playground 设计 §6 useNotes）：
 * manifest 是静态资产——无后端也可完整浏览；错误态由宿主渲染引导。
 */
export function useNotes(options: UseNotesOptions = {}) {
  const base = options.base ?? '/playground-data/notes.json'

  const manifest = shallowRef<NotesManifest | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const groups = computed<NoteGroup[]>(() => manifest.value?.groups ?? [])

  const flatNotes = computed<NoteWithGroup[]>(() =>
    groups.value.flatMap((group) => group.notes.map((note) => ({ note, group }))),
  )

  const byId = computed<Map<string, NoteWithGroup>>(() => {
    const map = new Map<string, NoteWithGroup>()
    for (const entry of flatNotes.value) map.set(entry.note.id, entry)
    return map
  })

  async function fetchNotes(url = base) {
    isLoading.value = true
    error.value = null
    try {
      const res = await fetch(url)
      if (!res.ok) throw new Error(`HTTP ${res.status} ${res.statusText}`)
      const data = JSON.parse(await res.text()) as NotesManifest
      if (!data || !Array.isArray(data.groups)) {
        throw new Error('invalid manifest: groups missing')
      }
      manifest.value = data
    } catch (e) {
      manifest.value = null
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      isLoading.value = false
    }
  }

  if (options.immediate !== false && typeof window !== 'undefined') {
    void fetchNotes()
  }

  return {
    /** 分组（manifest 序：order/id 已排序）。 */
    groups,
    /** 平铺笔记（附分组归属）。 */
    flatNotes,
    /** id → 笔记索引（深链定位）。 */
    byId,
    isLoading,
    error,
    fetchNotes,
  }
}
