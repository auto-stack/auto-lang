<template>
  <div class="app-shell">
    <header class="app-topbar">
      <span class="app-title">Auto Playground</span>
      <nav class="app-mode">
        <button class="mode-btn" :class="{ active: mode === 'notes' }" @click="mode = 'notes'">
          笔记站
        </button>
        <button class="mode-btn" :class="{ active: mode === 'ide' }" @click="mode = 'ide'">
          IDE 模式
        </button>
      </nav>
    </header>
    <main class="app-main">
      <NotesExplorer v-if="mode === 'notes'" ide-mode @ide-mode="onIdeMode" />
      <AutoPlaygroundFull v-else ref="ide" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue';
import { NotesExplorer, AutoPlaygroundFull } from 'auto-playground-vue';

type IdePayload = {
  noteId: string;
  source: string;
  projectDir?: string;
  files?: { path: string; source: string }[];
};

const mode = ref<'notes' | 'ide'>('notes');
// 结构化类型（InstanceType<typeof AutoPlaygroundFull> 在宿主 vue-tsc 触发过深类型实例化，同 P581-D2 家族限制）。
const ide = ref<{
  loadExample: (payload: { source: string; project_dir?: string; files?: { path: string; source: string }[] }) => void;
} | null>(null);

// "在 IDE 中打开"（Plan 582 T8/T14）：切换渲染全功能 IDE 并载入当前笔记。
function onIdeMode(payload: IdePayload) {
  mode.value = 'ide';
  void nextTick(() => {
    ide.value?.loadExample({
      source: payload.source,
      project_dir: payload.projectDir,
      files: payload.files,
    });
  });
}
</script>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #11111b;
}

.app-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.4rem 0.9rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.app-title {
  font-size: 0.9rem;
  font-weight: 700;
  font-family: 'JetBrains Mono', monospace;
  color: #cdd6f4;
}

.app-mode {
  display: flex;
  gap: 0.3rem;
}

.mode-btn {
  padding: 0.3rem 0.8rem;
  border: 1px solid #45475a;
  border-radius: 6px;
  background: transparent;
  color: #a6adc8;
  font-size: 0.78rem;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s, background 0.15s;
}

.mode-btn:hover {
  color: #cdd6f4;
}

.mode-btn.active {
  background: #6366f1;
  border-color: #6366f1;
  color: #fff;
}

.app-main {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
  padding: 0.5rem 0.9rem 1rem;
}
</style>
