<template>
  <div class="pgide-shell">
    <aside class="pgide-side">
      <NotesSidebar
        class="pgide-side-nav"
        :title="sidebarTitle"
        :groups="visibleGroups"
        :searching="!!query.trim()"
        :active-note-id="activeNoteId"
        v-model:query="query"
        @select="onSelect"
      />
    </aside>
    <main class="pgide-main">
      <AutoPlaygroundFull ref="ide" :note-meta="noteMeta" :api-base="apiBase" />
    </main>
  </div>
</template>

<script setup lang="ts">
// PLAN-754: 双端同形宿主——侧栏（分组树+搜索）+ AutoPlaygroundFull IDE。
// 独立版（backend 自服务 SPA）与 website /playground 共用本组件，
// 运行契约（timeout/prependLib 组判定）与 ?note= 深链内聚于此。
import { ref, computed, watch } from 'vue';
import NotesSidebar from './NotesSidebar.vue';
import AutoPlaygroundFull from '../AutoPlaygroundFull.vue';
import { useNotes } from '../composables/useNotes';
import type { NoteMeta } from '../types';

const props = withDefaults(
  defineProps<{
    /** 后端 API base（默认 '/api'）。website 宿主经探测传入。 */
    apiBase?: string;
    /** notes manifest URL（默认 /playground-data/notes.json，静态资产）。 */
    manifestBase?: string;
    sidebarTitle?: string;
  }>(),
  { apiBase: '/api', manifestBase: '/playground-data/notes.json', sidebarTitle: 'Auto Playground' },
);

const { groups, byId, search } = useNotes({ base: props.manifestBase });
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
  return {
    title: n.title,
    sourcePath: n.sourcePath,
    sourceType: n.sourceType,
    // PLAN-752: AAVM v1 自举语料（裸片段引用 lib-legacy 符号）需前置拼接。
    prependLib: n.id.startsWith('vm-bootstrap/'),
  };
});

// 结构化类型（InstanceType 触发宿主 vue-tsc 过深类型实例化，P581-D2 家族限制）。
const ide = ref<{
  loadExample: (payload: {
    source: string;
    project_dir?: string;
    files?: { path: string; source: string }[];
  }) => void;
} | null>(null);

// 首条笔记自动载入（manifest 到位后）。
function loadNote(note: NoteMeta) {
  activeNote.value = note;
  ide.value?.loadExample({
    source:
      note.code ??
      note.files?.find((f) => f.path === 'main.at')?.content ??
      note.files?.[0]?.content ??
      '',
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

// PLAN-754 G-3：?note=<id> 深链——打开即载对应笔记（可分享链接）。
// 无效 id 静默回退欢迎态（不做首条自动载入的 2026-09-07 复审裁定不受影响：
// 深链显式请求 ≠ 自动载入）。
// manifest 为异步 fetch——onMounted 时 byId 尚空，须在 manifest 到位后再尝试。
const deeplinkTried = ref(false);
watch(
  byId,
  () => {
    if (deeplinkTried.value) return;
    const id = new URLSearchParams(window.location.search).get('note');
    if (!id) {
      deeplinkTried.value = true;
      return;
    }
    const entry = byId.value.get(id);
    if (entry) {
      deeplinkTried.value = true;
      onSelect(id);
      history.replaceState(null, '', window.location.pathname);
    }
  },
  { immediate: true },
);
</script>

<style scoped>
.pgide-shell {
  display: flex;
  height: 100vh;
  background: #11111b;
  overflow: hidden;
}

.pgide-side {
  width: 280px;
  flex-shrink: 0;
  padding: 0.5rem 0 0.5rem 0.6rem;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 确定高度壳内侧栏占满全高（解除组件 76vh 上限）。 */
.pgide-side-nav {
  flex: 1 1 auto;
  min-height: 0;
  max-height: none;
}

.pgide-main {
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  padding: 0.5rem 0.6rem 0.5rem 0;
  display: flex;
  flex-direction: column;
}

/* IDE 直嵌占满主区（.playground 根为 100vh——壳无顶栏时恰为满高）。 */
.pgide-main :deep(.playground) {
  flex: 1 1 auto;
  min-height: 0;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid #313244;
}
</style>
