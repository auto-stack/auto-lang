<template>
  <aside class="nx-sidebar">
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
      <section v-for="g in groups" :key="g.id" class="nx-group">
        <button class="nx-group-head" @click="toggle(g.id)">
          <ChevronRight :size="13" class="nx-chev" :class="{ open: isOpen(g.id) }" />
          <span class="nx-group-title" :title="`${g.id} · ${g.source}`">{{ g.title }}</span>
          <span class="nx-count">{{ g.notes.length }}</span>
        </button>
        <ul v-show="isOpen(g.id)" class="nx-notes">
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
      <p v-if="groups.length === 0" class="nx-empty">无匹配笔记</p>
    </ScrollArea>
  </aside>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Search, ChevronRight } from 'lucide-vue-next'
import ScrollArea from './ScrollArea.vue'
import type { NoteGroup } from '../composables/useNotes'

const props = defineProps<{
  /** 展示分组（搜索态由父级传入过滤后的分组）。 */
  groups: NoteGroup[]
  /** 搜索进行中（强制展开命中分组）。 */
  searching: boolean
  /** 当前笔记 id。 */
  activeNoteId: string | null
}>()

const query = defineModel<string>('query', { default: '' })

const emit = defineEmits<{
  select: [noteId: string]
}>()

// 默认全折叠（VSCode 资源管理器惯例）；搜索态与活动笔记所在分组强制展开。
const openGroups = ref(new Set<string>())

function isOpen(id: string): boolean {
  if (props.searching) return true
  return openGroups.value.has(id)
}

function toggle(id: string) {
  const next = new Set(openGroups.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  openGroups.value = next
}

// 活动笔记变化时确保其所在分组展开并滚动进入视野。
watch(
  () => props.activeNoteId,
  (id) => {
    if (!id) return
    const group = props.groups.find((g) => g.notes.some((n) => n.id === id))
    if (group && !openGroups.value.has(group.id)) {
      const next = new Set(openGroups.value)
      next.add(group.id)
      openGroups.value = next
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
