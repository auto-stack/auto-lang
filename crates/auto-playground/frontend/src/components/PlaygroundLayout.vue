<template>
  <div class="playground">
    <header class="toolbar">
      <div class="toolbar-left">
        <h1 class="title">Auto Playground</h1>
        <ExampleSelector @select="onLoadExample" />
      </div>
      <div class="toolbar-right">
        <button class="share-btn" @click="$emit('share')" title="Copy shareable link">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/>
            <polyline points="16 6 12 2 8 6"/>
            <line x1="12" y1="2" x2="12" y2="15"/>
          </svg>
          Share
        </button>
      </div>
    </header>

    <div class="workspace">
      <div class="top-row">
        <div class="editor-pane">
          <div class="pane-header">Auto</div>
          <div class="pane-body">
            <CodeEditor
              :model-value="source"
              @update:model-value="$emit('update:source', $event)"
              :on-run="onRun"
              @line-click="$emit('lineClick', $event)"
            />
          </div>
        </div>
        <div class="transpile-pane">
          <OutputPanel
            :active-tab="activeTab"
            :transpiled-code="transpiledCode"
            :live-compile="liveCompile"
            :highlight-lines="highlightLines"
            @tab-change="onTabChange"
            @trans="$emit('trans')"
            @toggle-live="$emit('toggleLive')"
          />
        </div>
      </div>

      <div class="console-pane">
        <div class="pane-header">
          <span>Console</span>
          <button
            class="run-btn"
            @click="$emit('run')"
            :disabled="isLoading"
          >
            {{ isLoading ? 'Running...' : 'Run (Ctrl+Enter)' }}
          </button>
        </div>
        <div class="pane-body">
          <ConsoleOutput
            :stdout="stdout"
            :stderr="stderr"
            :result="resultCode"
            :time-ms="timeMs"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { OutputTab } from '../types';
import CodeEditor from './CodeEditor.vue';
import OutputPanel from './OutputPanel.vue';
import ConsoleOutput from './ConsoleOutput.vue';
import ExampleSelector from './ExampleSelector.vue';

defineProps<{
  source: string;
  isLoading: boolean;
  activeTab: OutputTab;
  stdout: string;
  stderr: string;
  resultCode: string;
  timeMs: number;
  transpiledCode: string;
  liveCompile: boolean;
  highlightLines?: number[];
  onRun: () => void;
}>();

const emit = defineEmits<{
  'update:source': [value: string];
  run: [];
  trans: [];
  tabChange: [tab: OutputTab];
  loadExample: [code: string];
  toggleLive: [];
  lineClick: [line: number];
  share: [];
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
  flex-shrink: 0;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
}
.toolbar-right {
  display: flex;
  gap: 8px;
  align-items: center;
}
.title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: #fff;
}
.share-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: #3c3c3c;
  color: #ccc;
  border: 1px solid #555;
  border-radius: 4px;
  padding: 6px 14px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: background 0.15s;
}
.share-btn:hover {
  background: #4a4a4a;
  color: #fff;
}

.workspace {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.top-row {
  flex: 2;
  display: flex;
  overflow: hidden;
}
.editor-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #444;
  overflow: hidden;
}
.transpile-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.console-pane {
  flex: 1;
  min-height: 140px;
  max-height: 45%;
  display: flex;
  flex-direction: column;
  border-top: 1px solid #444;
  overflow: hidden;
}

.pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  flex-shrink: 0;
}
.pane-body {
  flex: 1;
  overflow: hidden;
}

.run-btn {
  background: #0e639c;
  color: #fff;
  border: none;
  border-radius: 4px;
  padding: 4px 14px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: background 0.15s;
}
.run-btn:hover {
  background: #1177bb;
}
.run-btn:disabled {
  background: #555;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .top-row {
    flex-direction: column;
  }
  .editor-pane {
    border-right: none;
    border-bottom: 1px solid #444;
  }
}
</style>
