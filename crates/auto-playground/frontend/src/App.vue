<template>
  <div class="app-shell">
    <aside class="app-side">
      <NotesSidebar
        class="app-side-nav"
        title="Auto Playground"
        :groups="visibleGroups"
        :searching="!!query.trim()"
        :active-note-id="activeNoteId"
        v-model:query="query"
        @select="onSelect"
      />
    </aside>
    <main class="app-main">
      <AutoPlaygroundFull ref="ide" :note-meta="noteMeta" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { NotesSidebar, AutoPlaygroundFull, useNotes } from 'auto-playground-vue';
import type { NoteMeta } from 'auto-playground-vue';

const { groups, flatNotes, byId, search } = useNotes();
const query = ref('');

// 活动笔记（侧栏点击切换；标题栏元信息 + IDE 载入）。
const activeNoteId = ref<string | null>(null);
const activeNote = ref<NoteMeta | null>(null);

const visibleGroups = computed(() => {
  if (!query.value.trim()) return groups.value;
  const byGroup = new Map<string, NoteMeta[]>();
  for (const { note, group } of search(query.value)) {
    if (!byGroup.has(group.id)) byGroup.set(group.id, []);
    byGroup.get(group.id)!.push(note);
  }
  return groups.value
    .map((g) => ({ ...g, notes: byGroup.get(g.id) ?? [] }))
    .filter((g) => g.notes.length > 0);
});

const noteMeta = computed(() => {
  const n = activeNote.value;
  if (!n) return null;
  return { title: n.title, sourcePath: n.sourcePath, sourceType: n.sourceType };
});

// 结构化类型（InstanceType 触发宿主 vue-tsc 过深类型实例化，P581-D2 家族限制）。
const ide = ref<{
  loadExample: (payload: { source: string; project_dir?: string; files?: { path: string; source: string }[] }) => void;
} | null>(null);

// 首条笔记自动载入（manifest 到位后）。
function loadNote(note: NoteMeta) {
  activeNote.value = note;
  ide.value?.loadExample({
    source: note.code ?? note.files?.find((f) => f.path === 'main.at')?.content ?? '',
    project_dir: projectDirOf(note),
    files: note.files?.map((f) => ({ path: f.path, source: f.content })),
  });
}

function projectDirOf(note: NoteMeta): string | undefined {
  if (note.kind !== 'project' || !note.files) return undefined;
  const m = note.sourcePath.match(/examples\/playground-demo\/(.+)\/main\.at$/);
  return m ? m[1] : undefined;
}

function onSelect(noteId: string) {
  const entry = byId.value.get(noteId);
  if (!entry) return;
  activeNoteId.value = noteId;
  loadNote(entry.note);
}

// manifest 到位后自动载入首条（深链/记忆状态不做——后端壳定位为浏览入口）。
watch(() => flatNotes.value, (notes) => {
  if (!activeNoteId.value && notes.length > 0) {
    onSelect(notes[0]!.note.id);
  }
});
</script>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  background: #11111b;
  overflow: hidden;
}

.app-side {
  width: 280px;
  flex-shrink: 0;
  padding: 0.5rem 0 0.5rem 0.6rem;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 确定高度壳内侧栏占满全高（解除组件 76vh 上限）。 */
.app-side-nav {
  flex: 1 1 auto;
  min-height: 0;
  max-height: none;
}

.app-main {
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  padding: 0.5rem 0.6rem 0.5rem 0;
  display: flex;
  flex-direction: column;
}

/* IDE 直嵌占满主区（.playground 根为 100vh——壳无顶栏时恰为满高）。 */
.app-main :deep(.playground) {
  flex: 1 1 auto;
  min-height: 0;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid #313244;
}
</style>
