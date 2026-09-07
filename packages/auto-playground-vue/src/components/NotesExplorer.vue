<template>
  <div class="notes-explorer">
    <NotesSidebar
      class="nx-side"
      :groups="visibleGroups"
      :searching="!!query.trim()"
      :active-note-id="activeId"
      v-model:query="query"
      @select="onSelect"
    />
    <main class="nx-main">
      <div v-if="isLoading" class="nx-state">正在加载笔记清单…</div>
      <div v-else-if="error" class="nx-state nx-error">
        <p>笔记清单加载失败：{{ error }}</p>
        <button class="nx-retry" @click="fetchNotes()">
          <RefreshCw :size="13" /> 重试
        </button>
      </div>
      <div v-else-if="flatNotes.length === 0" class="nx-state">笔记清单为空。</div>
      <div v-else-if="!active" class="nx-state">从左侧选择一条笔记开始浏览。</div>
      <article v-else class="nx-note">
        <header class="nx-note-header">
          <div class="nx-title-row">
            <h2 class="nx-note-title">{{ active.note.title }}</h2>
            <span class="nx-badge" :class="`t-${active.note.sourceType}`">{{ active.note.sourceType }}</span>
            <span v-if="active.note.kind !== 'single'" class="nx-kind-chip">{{ kindLabel }}</span>
          </div>
          <div class="nx-meta-row">
            <a class="nx-source-chip" :href="sourceUrl" target="_blank" rel="noopener" :title="active.note.sourcePath">
              <FileCode :size="12" />
              <span class="nx-source-path">{{ active.note.sourcePath }}</span>
              <ExternalLink :size="11" />
            </a>
            <span v-if="!active.note.standalone" class="nx-standalone-warn" title="含 import/use 顶层声明，不可独立运行">
              依赖模块
            </span>
          </div>
          <details v-if="active.note.description" class="nx-desc">
            <summary>说明</summary>
            <p>{{ active.note.description }}</p>
          </details>
        </header>
        <PlaygroundCard
          ref="card"
          :key="active.note.id"
          :code="cardCode"
          :api-base="apiBase"
          :note-id="active.note.id"
          :expected-output="active.note.expectedOutput"
          :files="active.note.files"
          :project-dir="cardProjectDir"
          height="auto"
        />
      </article>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { FileCode, ExternalLink, RefreshCw } from 'lucide-vue-next'
import NotesSidebar from './NotesSidebar.vue'
import PlaygroundCard from './PlaygroundCard.vue'
import { useNotes } from '../composables/useNotes'
import type { NoteGroup } from '../composables/useNotes'
import type { NoteMeta } from '../types'

const props = withDefaults(defineProps<{
  /** manifest URL；默认 /playground-data/notes.json（SPA 宿主经参数改 base）。 */
  base?: string
  /** 后端地址；'' = 同源 /api。 */
  apiBase?: string
  /** 来源 chip 的仓库根（GitHub blob 链接前缀）。 */
  repoBase?: string
}>(), {
  base: '/playground-data/notes.json',
  apiBase: '',
  repoBase: 'https://github.com/auto-stack/auto-lang',
})

const { groups, flatNotes, byId, isLoading, error, fetchNotes, search } = useNotes({ base: props.base })

const query = ref('')
const activeId = ref<string | null>(null)

const card = ref<InstanceType<typeof PlaygroundCard> | null>(null)

const active = computed(() => (activeId.value ? byId.value.get(activeId.value) ?? null : null))

// 搜索态：命中笔记按原分组收拢（分组归属保留，Playground 设计 §6.1）。
const visibleGroups = computed<NoteGroup[]>(() => {
  if (!query.value.trim()) return groups.value
  const byGroup = new Map<string, NoteMeta[]>()
  for (const { note, group } of search(query.value)) {
    if (!byGroup.has(group.id)) byGroup.set(group.id, [])
    byGroup.get(group.id)!.push(note)
  }
  return groups.value
    .map((g) => ({ ...g, notes: byGroup.get(g.id) ?? [] }))
    .filter((g) => g.notes.length > 0)
})

const sourceUrl = computed(() => {
  const n = active.value?.note
  if (!n) return ''
  return `${props.repoBase.replace(/\/$/, '')}/blob/master/${n.sourcePath}`
})

const KIND_LABELS: Record<string, string> = {
  single: '单文件',
  project: '项目',
  fence: '书页围栏',
}

const kindLabel = computed(() => KIND_LABELS[active.value?.note.kind ?? ''] ?? active.value?.note.kind ?? '')

// T7 接项目型文件 tab；骨架期 project 笔记以 main.at 内容起步。
const cardCode = computed(() => {
  const n = active.value?.note
  if (!n) return ''
  return n.code ?? n.files?.find((f) => f.path === 'main.at')?.content ?? ''
})

// 项目目录（相对服务端 examples/playground-demo；运行 files 形态的物化基座）。
const cardProjectDir = computed(() => {
  const n = active.value?.note
  if (!n || n.kind !== 'project' || !n.files) return null
  const m = n.sourcePath.match(/examples\/playground-demo\/(.+)\/main\.at$/)
  return m ? m[1] : null
})

function onSelect(noteId: string) {
  activeId.value = noteId
}

// ── 深链 #/notes/<id>（hash 变化不触发 VitePress 路由——replaceState 不派发路由事件）──

const HASH_PREFIX = '#/notes/'

function noteIdFromHash(): string | null {
  if (typeof window === 'undefined') return null
  const h = window.location.hash
  if (!h.startsWith(HASH_PREFIX)) return null
  try {
    return decodeURIComponent(h.slice(HASH_PREFIX.length))
  } catch {
    return null
  }
}

function writeHash(noteId: string) {
  if (typeof window === 'undefined') return
  // '/' 在 hash 内合法，保持 id 可读；其余特殊字符编码。
  const want = HASH_PREFIX + encodeURIComponent(noteId).replace(/%2F/gi, '/')
  if (window.location.hash !== want) {
    window.history.replaceState(null, '', want)
  }
}

watch(flatNotes, (notes) => {
  if (activeId.value || notes.length === 0) return
  const fromHash = noteIdFromHash()
  activeId.value = fromHash && byId.value.has(fromHash) ? fromHash : notes[0].note.id
})

watch(activeId, (id) => {
  if (id) writeHash(id)
})

function onHashChange() {
  const id = noteIdFromHash()
  if (id && byId.value.has(id) && id !== activeId.value) {
    activeId.value = id
  }
}

// ── 键盘导航：↑/↓ 切换当前笔记（搜索态沿命中列表），Ctrl+Enter 运行（转发 PlaygroundCard）──

function isEditableTarget(target: EventTarget | null): boolean {
  const el = target instanceof HTMLElement ? target : null
  if (!el) return false
  return !!el.closest('input, textarea, select, [contenteditable="true"], .cm-editor')
}

function visibleNoteIds(): string[] {
  return visibleGroups.value.flatMap((g) => g.notes.map((n) => n.id))
}

function moveSelection(delta: number) {
  const ids = visibleNoteIds()
  if (ids.length === 0) return
  const idx = activeId.value ? ids.indexOf(activeId.value) : -1
  const next = idx === -1 ? 0 : Math.min(ids.length - 1, Math.max(0, idx + delta))
  activeId.value = ids[next]
}

function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    e.preventDefault()
    void card.value?.run()
    return
  }
  if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
    if (isEditableTarget(e.target)) return
    e.preventDefault()
    moveSelection(e.key === 'ArrowDown' ? 1 : -1)
  }
}

onMounted(() => {
  window.addEventListener('hashchange', onHashChange)
  window.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('hashchange', onHashChange)
  window.removeEventListener('keydown', onKeydown)
})
</script>

<!-- CSS 变量消费 VitePress 主题 token，包内 fallback 适配无 VitePress 变量的 SPA 宿主（Playground 设计 §6）。 -->
<style scoped>
.notes-explorer {
  --nx-border: var(--vp-c-border, #313244);
  --nx-bg: var(--vp-c-bg, #1e1e1e);
  --nx-bg-alt: var(--vp-c-bg-alt, #181825);
  --nx-bg-soft: var(--vp-c-bg-soft, rgba(128, 128, 128, 0.14));
  --nx-text-1: var(--vp-c-text-1, #cdd6f4);
  --nx-text-2: var(--vp-c-text-2, #a6adc8);
  --nx-text-3: var(--vp-c-text-3, #6c7086);
  --nx-brand: var(--vp-c-brand-1, #6366f1);
  --nx-brand-text: var(--vp-c-brand-1, #a5b4fc);
  --nx-brand-soft: var(--vp-c-brand-soft, rgba(99, 102, 241, 0.16));
  --nx-green: var(--vp-c-green-1, #27c93f);
  --nx-red: var(--vp-c-red-1, #f38ba8);
  --nx-yellow: var(--vp-c-yellow-1, #f9e2af);
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
  gap: 1rem;
  margin: 1rem 0;
}

.nx-main {
  min-width: 0;
}

.nx-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.6rem;
  padding: 3rem 1rem;
  border: 1px dashed var(--nx-border);
  border-radius: 10px;
  color: var(--nx-text-3);
  font-size: 0.85rem;
}

.nx-state p {
  margin: 0;
}

.nx-retry {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.35rem 0.8rem;
  border: 1px solid var(--nx-border);
  border-radius: 6px;
  background: var(--nx-bg-alt);
  color: var(--nx-text-1);
  font-size: 0.8rem;
  cursor: pointer;
}

.nx-retry:hover {
  border-color: var(--nx-brand);
}

.nx-note {
  min-width: 0;
}

.nx-note-header {
  margin-bottom: 0.75rem;
}

.nx-title-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  flex-wrap: wrap;
}

.nx-note-title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 700;
  color: var(--nx-text-1);
}

.nx-badge {
  font-size: 0.68rem;
  font-family: 'JetBrains Mono', monospace;
  padding: 0.1rem 0.5rem;
  border-radius: 9px;
  border: 1px solid var(--nx-border);
  color: var(--nx-text-2);
  background: var(--nx-bg-alt);
}

.nx-badge.t-vm-golden {
  color: var(--nx-yellow);
  border-color: color-mix(in srgb, var(--nx-yellow) 40%, transparent);
}

.nx-badge.t-aavm-corpus {
  color: var(--nx-brand-text);
  border-color: color-mix(in srgb, var(--nx-brand) 40%, transparent);
}

.nx-badge.t-book {
  color: var(--nx-green);
  border-color: color-mix(in srgb, var(--nx-green) 40%, transparent);
}

.nx-kind-chip {
  font-size: 0.68rem;
  padding: 0.1rem 0.5rem;
  border-radius: 9px;
  color: var(--nx-text-3);
  border: 1px dashed var(--nx-border);
}

.nx-meta-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-top: 0.4rem;
  flex-wrap: wrap;
}

.nx-source-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  max-width: 100%;
  padding: 0.15rem 0.55rem;
  border-radius: 6px;
  border: 1px solid var(--nx-border);
  background: var(--nx-bg-alt);
  color: var(--nx-text-2);
  font-size: 0.72rem;
  font-family: 'JetBrains Mono', monospace;
  text-decoration: none;
}

.nx-source-chip:hover {
  color: var(--nx-brand-text);
  border-color: var(--nx-brand);
}

.nx-source-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.nx-standalone-warn {
  font-size: 0.7rem;
  color: var(--nx-yellow);
}

.nx-desc {
  margin-top: 0.5rem;
  font-size: 0.8rem;
  color: var(--nx-text-2);
  border: 1px solid var(--nx-border);
  border-radius: 8px;
  background: var(--nx-bg-alt);
  padding: 0.35rem 0.7rem;
}

.nx-desc summary {
  cursor: pointer;
  color: var(--nx-text-3);
  font-weight: 600;
  user-select: none;
}

.nx-desc p {
  margin: 0.35rem 0 0.2rem;
}

@media (max-width: 960px) {
  .notes-explorer {
    grid-template-columns: 1fr;
  }

  .notes-explorer :deep(.nx-tree) {
    max-height: 40vh;
  }
}
</style>
