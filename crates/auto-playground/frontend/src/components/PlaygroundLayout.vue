<template>
  <div class="playground">
    <header class="toolbar">
      <div class="toolbar-left">
        <h1 class="title">Auto Playground</h1>
        <ExampleSelector @select="onLoadExample" />
      </div>
      <div class="toolbar-right">
        <button
          class="run-btn"
          @click="$emit('run')"
          :disabled="isLoading"
        >
          {{ isLoading ? 'Running...' : 'Run (Ctrl+Enter)' }}
        </button>
      </div>
    </header>
    <main class="main">
      <div class="editor-pane">
        <CodeEditor
          :model-value="source"
          @update:model-value="$emit('update:source', $event)"
          :on-run="onRun"
        />
      </div>
      <div class="output-pane">
        <OutputPanel
          :active-tab="activeTab"
          :stdout="stdout"
          :stderr="stderr"
          :time-ms="timeMs"
          :transpiled-code="transpiledCode"
          :live-compile="liveCompile"
          @tab-change="onTabChange"
          @toggle-live="$emit('toggleLive')"
        />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import type { OutputTab } from '../types';
import CodeEditor from './CodeEditor.vue';
import OutputPanel from './OutputPanel.vue';
import ExampleSelector from './ExampleSelector.vue';

defineProps<{
  source: string;
  isLoading: boolean;
  activeTab: OutputTab;
  stdout: string;
  stderr: string;
  timeMs: number;
  transpiledCode: string;
  liveCompile: boolean;
  onRun: () => void;
}>();

const emit = defineEmits<{
  'update:source': [value: string];
  run: [];
  tabChange: [tab: OutputTab];
  loadExample: [code: string];
  toggleLive: [];
}>();

function onTabChange(tab: OutputTab) {
  emit('tabChange', tab);
}

function onLoadExample(code: string) {
  emit('loadExample', code);
}
</script>

<style scoped>
.playground {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #1e1e1e;
  color: #d4d4d4;
}
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
}
.toolbar-right {
  display: flex;
  gap: 8px;
}
.title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: #fff;
}
.run-btn {
  background: #0e639c;
  color: #fff;
  border: none;
  border-radius: 4px;
  padding: 6px 16px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}
.run-btn:hover {
  background: #1177bb;
}
.run-btn:disabled {
  background: #555;
  cursor: not-allowed;
}
.main {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.editor-pane {
  flex: 1;
  border-right: 1px solid #444;
}
.output-pane {
  flex: 1;
}

@media (max-width: 768px) {
  .main {
    flex-direction: column;
  }
  .editor-pane {
    border-right: none;
    border-bottom: 1px solid #444;
  }
}
</style>
