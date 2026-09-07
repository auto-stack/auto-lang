<template>
  <aside class="nx-sidebar">
    <div v-if="title" class="nx-brand">
      <Zap :size="16" class="nx-brand-icon" />
      <span>{{ title }}</span>
    </div>
    <div class="nx-search">
      <Search :size="14" class="nx-search-icon" />
      <input
        class="nx-search-input"
        type="text"
        placeholder="搜索标题或标签…"
        :value="query"
        @input="query = ($event.target as HTMLInputElement).value"
      />
    </div>
    <ScrollArea class="nx-tree">
      <section v-for="sec in sections" :key="sec.id" class="nx-group">
        <button class="nx-group-head" @click="toggleSection(sec.id)">
          <ChevronRight :size="13" class="nx-chev" :class="{ open: isSectionOpen(sec.id) }" />
          <span class="nx-group-title" :title="sec.title">{{ sec.title }}</span>
          <span class="nx-count">{{ sec.noteCount }}</span>
        </button>
        <div v-show="isSectionOpen(sec.id)" class="nx-section-body">
          <!-- Playground Demo：单组，笔记直接展开 -->
          <template v-if="sec.id === 'demo'">
            <ul class="nx-notes">
              <li v-for="n in sec.groups[0]?.notes ?? []" :key="n.id">
                <button
                  class="nx-note-btn"
                  :class="{ active: n.id === activeNoteId }"
                  :title="n.id"
                  @click="emit('select', n.id)"
                >
                  {{ n.title }}
                </button>
              </li>
            </ul>
          </template>
          <!-- 书籍示例：书（用户指定序）→ 章节目录 → 笔记（Plan 582 复审修正） -->
          <template v-else-if="sec.id === 'books'">
            <section v-for="g in sec.groups" :key="g.id" class="nx-group nx-subgroup">
              <button class="nx-group-head" @click="toggle(g.id)" :title="`${g.id} · ${g.source}`">
                <ChevronRight :size="13" class="nx-chev" :class="{ open: isOpen(g.id) }" />
                <span class="nx-group-title">{{ bookTitle(g.id) }}</span>
                <span class="nx-count">{{ g.notes.length }}</span>
              </button>
              <div v-show="isOpen(g.id)" class="nx-section-body nx-chapters">
                <section v-for="ch in chaptersOf(g)" :key="ch.key" class="nx-group nx-subgroup nx-chapter">
                  <button class="nx-group-head" @click="toggleChapter(ch.key)" :title="ch.key">
                    <ChevronRight :size="13" class="nx-chev" :class="{ open: isChapterOpen(ch.key) }" />
                    <span class="nx-group-title">{{ ch.stem }}</span>
                    <span class="nx-count">{{ ch.notes.length }}</span>
                  </button>
                  <ul v-show="isChapterOpen(ch.key)" class="nx-notes nx-notes-chapter">
                    <li v-for="n in ch.notes" :key="n.id">
                      <button
                        class="nx-note-btn"
                        :class="{ active: n.id === activeNoteId }"
                        :title="n.id"
                        @click="emit('select', n.id)"
                      >
                        {{ bookNoteLabel(n, ch.stem) }}
                      </button>
                    </li>
                  </ul>
                </section>
              </div>
            </section>
          </template>
          <!-- 测试用例：子目录（组）层 -->
          <template v-else>
            <section v-for="g in sec.groups" :key="g.id" class="nx-group nx-subgroup">
              <button class="nx-group-head" @click="toggle(g.id)" :title="`${g.id} · ${g.source}`">
                <ChevronRight :size="13" class="nx-chev" :class="{ open: isOpen(g.id) }" />
                <span class="nx-group-title">{{ g.title }}</span>
                <span class="nx-count">{{ g.notes.length }}</span>
              </button>
              <ul v-show="isOpen(g.id)" class="nx-notes nx-notes-sub">
                <li v-for="n in g.notes" :key="n.id">
                  <button
                    class="nx-note-btn"
                    :class="{ active: n.id === activeNoteId }"
                    :title="n.id"
                    @click="emit('select', n.id)"
                  >
                    {{ n.title }}
                  </button>
                </li>
              </ul>
            </section>
          </template>
        </div>
      </section>
      <p v-if="groups.length === 0" class="nx-empty">无匹配笔记</p>
    </ScrollArea>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Search, ChevronRight, Zap } from 'lucide-vue-next'
import ScrollArea from './ScrollArea.vue'
import type { NoteGroup } from '../composables/useNotes'
import type { NoteMeta } from '../types'

const props = defineProps<{
  /** 展示分组（搜索态由父级传入过滤后的分组）。 */
  groups: NoteGroup[]
  /** 搜索进行中（强制展开命中分组）。 */
  searching: boolean
  /** 当前笔记 id。 */
  activeNoteId: string | null
  /** 品牌标题（可选；渲染在搜索框上方，后端壳注入）。 */
  title?: string
}>()

const query = defineModel<string>('query', { default: '' })

const emit = defineEmits<{
  select: [noteId: string]
}>()

// ── 三级归类（用户裁定，Plan 582 复审修正）：Demo / 书籍示例 / 测试用例 ──
// manifest 保持平铺（groups+sourceType 单一事实源不变），归类是纯展示层推导。

interface TreeSection {
  id: 'demo' | 'books' | 'tests'
  title: string
  groups: NoteGroup[]
  noteCount: number
}

const sections = computed<TreeSection[]>(() => {
  const demo = props.groups.filter((g) => g.id === 'demo')
  const books = props.groups
    .filter((g) => g.id.startsWith('book-'))
    .sort((a, b) => bookRank(a.id) - bookRank(b.id))
  const tests = props.groups.filter((g) => g.id !== 'demo' && !g.id.startsWith('book-'))
  const out: TreeSection[] = []
  const push = (id: TreeSection['id'], title: string, gs: NoteGroup[]) => {
    if (gs.length === 0) return
    out.push({ id, title, groups: gs, noteCount: gs.reduce((s, g) => s + g.notes.length, 0) })
  }
  push('demo', 'Playground Demo', demo)
  push('books', '书籍示例', books)
  push('tests', '测试用例', tests)
  return out
})

// ── 书籍展示序与命名（用户裁定：tapl=官方书第一，其后 Rust/Think Python/…）──

const BOOK_ORDER = [
  'book-tapl',
  'book-rust',
  'book-think-python',
  'book-byte-of-python',
  'book-typescript',
  'book-typescript-deepdive',
  'book-little-c',
  'book-modern-c',
]

const BOOK_TITLES: Record<string, string> = {
  'book-tapl': 'TAPL（官方书籍）',
  'book-rust': 'Rust',
  'book-think-python': 'Think Python',
  'book-byte-of-python': 'Byte of Python',
  'book-typescript': 'Typescript',
  'book-typescript-deepdive': 'Typescript Deepdive',
  'book-little-c': 'Little C',
  'book-modern-c': 'Modern C',
}

function bookRank(id: string): number {
  const i = BOOK_ORDER.indexOf(id)
  return i === -1 ? BOOK_ORDER.length : i
}

function bookTitle(id: string): string {
  return BOOK_TITLES[id] ?? id.replace(/^book-/, '')
}

// 章节推导：书内笔记 id 形如 `book-X/<chNN-slug>-<idx>`，按 stem 聚合成章节目录。
interface BookChapter {
  key: string
  stem: string
  notes: NoteMeta[]
}

function chapterStemOf(group: NoteGroup, noteId: string): string {
  const rest = noteId.slice(group.id.length + 1)
  return rest.replace(/-\d+$/, '')
}

function chaptersOf(group: NoteGroup): BookChapter[] {
  const byStem = new Map<string, NoteMeta[]>()
  for (const note of group.notes) {
    const stem = chapterStemOf(group, note.id)
    if (!byStem.has(stem)) byStem.set(stem, [])
    byStem.get(stem)!.push(note)
  }
  return Array.from(byStem.entries())
    .sort(([a], [b]) => a.localeCompare(b, undefined, { numeric: true }))
    .map(([stem, notes]) => ({ key: `${group.id}/${stem}`, stem, notes }))
}

// 书内笔记在章节层下的展示名：剥掉标题中的「<stem> · 」前缀（章节名已在上级目录）。
function bookNoteLabel(note: NoteMeta, stem: string): string {
  const prefix = `${stem} · `
  return note.title.startsWith(prefix) ? note.title.slice(prefix.length) : note.title
}

// 展开态：section（第一层）与组（第二层）各自独立；章节（书籍第三层）键为
// `<groupId>/<stem>`；搜索态强制全开。
const openSections = ref(new Set<string>())
const openGroups = ref(new Set<string>())
const openChapters = ref(new Set<string>())

function isSectionOpen(id: string): boolean {
  return props.searching || openSections.value.has(id)
}

function isOpen(id: string): boolean {
  return props.searching || openGroups.value.has(id)
}

function isChapterOpen(key: string): boolean {
  return props.searching || openChapters.value.has(key)
}

function toggleIn(set: Set<string>, id: string) {
  const next = new Set(set)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  return next
}

function toggleSection(id: string) {
  openSections.value = toggleIn(openSections.value, id)
}

function toggle(id: string) {
  openGroups.value = toggleIn(openGroups.value, id)
}

function toggleChapter(key: string) {
  openChapters.value = toggleIn(openChapters.value, key)
}

function sectionOfGroup(groupId: string): string | null {
  if (groupId === 'demo') return 'demo'
  if (groupId.startsWith('book-')) return 'books'
  return groupId ? 'tests' : null
}

// 活动笔记变化：展开其所属 section + 组（书籍再多展开章节层），并滚动进入视野。
watch(
  () => props.activeNoteId,
  (id) => {
    if (!id) return
    const group = props.groups.find((g) => g.notes.some((n) => n.id === id))
    if (!group) return
    const sectionId = sectionOfGroup(group.id)
    if (sectionId && !openSections.value.has(sectionId)) {
      openSections.value = toggleIn(openSections.value, sectionId)
    }
    if (sectionId !== 'demo' && !openGroups.value.has(group.id)) {
      openGroups.value = toggleIn(openGroups.value, group.id)
    }
    if (sectionId === 'books') {
      const stem = chapterStemOf(group, id)
      const key = `${group.id}/${stem}`
      if (!openChapters.value.has(key)) {
        openChapters.value = toggleIn(openChapters.value, key)
      }
    }
    requestAnimationFrame(() => {
      document.querySelector('.nx-note-btn.active')?.scrollIntoView({ block: 'nearest' })
    })
  },
)
</script>

<style scoped>
.nx-sidebar {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--nx-border);
  border-radius: 10px;
  background: var(--nx-bg);
  overflow: hidden;
  /* 高度上限给内部 ScrollArea 确定的高度上下文（浮动滚动条需要）；内容少时随内容收缩。 */
  max-height: min(76vh, 900px);
}

.nx-brand {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.6rem 0.7rem 0.45rem;
  font-size: 1.05rem;
  font-weight: 700;
  font-family: 'JetBrains Mono', monospace;
  color: #818cf8;
  border-bottom: 1px solid var(--nx-border);
  flex-shrink: 0;
}

.nx-brand-icon {
  color: #863bff;
  fill: #863bff;
  flex-shrink: 0;
}

.nx-search {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.6rem;
  border-bottom: 1px solid var(--nx-border);
  background: var(--nx-bg-alt);
  flex-shrink: 0;
}

.nx-search-icon {
  color: var(--nx-text-3);
  flex-shrink: 0;
}

.nx-search-input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  color: var(--nx-text-1);
  font-size: 0.8rem;
}

.nx-search-input::placeholder {
  color: var(--nx-text-3);
}

.nx-tree {
  flex: 1 1 auto;
  min-height: 0;
  padding: 0.35rem 0.25rem;
}

.nx-group {
  margin-bottom: 0.15rem;
}

.nx-group-head {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  width: 100%;
  padding: 0.3rem 0.4rem;
  background: none;
  border: none;
  border-radius: 6px;
  color: var(--nx-text-2);
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  text-align: left;
}

.nx-group-head:hover {
  background: var(--nx-bg-soft);
  color: var(--nx-text-1);
}

.nx-chev {
  flex-shrink: 0;
  transition: transform 0.15s;
  color: var(--nx-text-3);
}

.nx-chev.open {
  transform: rotate(90deg);
}

.nx-group-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.nx-count {
  flex-shrink: 0;
  font-size: 0.68rem;
  font-family: 'JetBrains Mono', monospace;
  color: var(--nx-text-3);
  background: var(--nx-bg-soft);
  border-radius: 8px;
  padding: 0 0.45rem;
}

.nx-notes {
  list-style: none;
  margin: 0.1rem 0 0.25rem;
  padding: 0 0 0 1.1rem;
}

/* 第二层 section 体内的子目录缩进与笔记再缩进 */
.nx-section-body {
  padding-left: 0.35rem;
}

.nx-subgroup > .nx-group-head {
  padding-left: 0.9rem;
}

.nx-notes-sub {
  padding-left: 1.7rem;
}

.nx-note-btn {
  display: block;
  width: 100%;
  padding: 0.22rem 0.45rem;
  background: none;
  border: none;
  border-radius: 5px;
  color: var(--nx-text-2);
  font-size: 0.78rem;
  cursor: pointer;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.nx-note-btn:hover {
  background: var(--nx-bg-soft);
  color: var(--nx-text-1);
}

.nx-note-btn.active {
  background: var(--nx-brand-soft);
  color: var(--nx-brand-text);
}

.nx-empty {
  padding: 0.75rem;
  margin: 0;
  color: var(--nx-text-3);
  font-size: 0.78rem;
  text-align: center;
}
</style>
