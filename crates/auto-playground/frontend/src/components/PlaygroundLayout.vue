<template>
  <div class="playground">
    <header class="toolbar">
      <div class="toolbar-left">
        <h1 class="title">Auto Playground</h1>
        <ExampleSelector @select="onLoadExample" />
      </div>
      <div class="toolbar-right">
        <button
          :class="['debug-btn', { active: isDebugging }]"
          @click="$emit('toggleDebug')"
          :title="isDebugging ? 'Stop Debugging' : 'Start Debugging'"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2a10 10 0 0 1 10 10"/>
            <path d="M12 2a10 10 0 0 0-10 10"/>
            <path d="M12 12l4-4"/>
            <path d="M12 12l-4-4"/>
            <path d="M12 12l4 4"/>
            <path d="M12 12l-4 4"/>
          </svg>
          {{ isDebugging ? 'Stop' : 'Debug' }}
        </button>
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

    <DebugToolbar
      v-if="isDebugging"
      :is-paused="isPaused"
      @command="$emit('debugCommand', $event)"
    />

    <div class="workspace">
      <div class="top-row">
        <div class="editor-pane">
          <div class="pane-header">Auto</div>
          <div class="pane-body">
            <CodeEditor
              :model-value="source"
              @update:model-value="$emit('update:source', $event)"
              :on-run="onRun"
              :is-debugging="isDebugging"
              :breakpoints="breakpoints"
              :current-debug-line="currentDebugLine"
              @line-click="$emit('lineClick', $event)"
              @breakpoints-change="$emit('breakpointsChange', $event)"
            />
          </div>
        </div>
        <div class="transpile-pane">
          <OutputPanel
            :active-tab="activeTab"
            :transpiled-code="transpiledCode"
            :live-compile="liveCompile"
            :highlight-lines="highlightLines"
            :bytecode="bytecode"
            :current-ip="debugState?.ip"
            :highlighted-offsets="highlightedOffsets"
            @tab-change="onTabChange"
            @trans="$emit('trans')"
            @toggle-live="$emit('toggleLive')"
            @offset-click="$emit('offsetClick', $event)"
          />
        </div>
      </div>

      <div class="console-pane">
        <div class="pane-header">
          <div class="console-tabs">
            <button
              :class="{ active: consoleTab === 'output' }"
              @click="consoleTab = 'output'"
            >Console</button>
            <button
              v-if="isDebugging"
              :class="{ active: consoleTab === 'debug' }"
              @click="consoleTab = 'debug'"
            >Debug</button>
          </div>
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
            v-if="consoleTab === 'output'"
            :stdout="stdout"
            :stderr="stderr"
            :result="resultCode"
            :time-ms="timeMs"
          />
          <DebugPanel
            v-else-if="consoleTab === 'debug' && debugState"
            :state="debugState"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { OutputTab, BytecodeLine, DebugState } from '../types';
import CodeEditor from './CodeEditor.vue';
import OutputPanel from './OutputPanel.vue';
import ConsoleOutput from './ConsoleOutput.vue';
import ExampleSelector from './ExampleSelector.vue';
import DebugToolbar from './DebugToolbar.vue';
import DebugPanel from './DebugPanel.vue';

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
  // Debug props
  isDebugging?: boolean;
  isPaused?: boolean;
  bytecode?: BytecodeLine[];
  debugState?: DebugState | null;
  currentSourceLine?: number | null;
  highlightedOffsets?: number[];
  breakpoints?: number[];
  currentDebugLine?: number | null;
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
  // Debug events
  toggleDebug: [];
  debugCommand: [cmd: 'continue' | 'step' | 'step_over' | 'step_out' | 'stop'];
  offsetClick: [offset: number];
  breakpointsChange: [lines: number[]];
}>();

function onTabChange(tab: OutputTab) {
  emit('tabChange', tab);
}

function onLoadExample(code: string) {
  emit('loadExample', code);
}

const consoleTab = ref<'output' | 'debug'>('output');
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
.debug-btn {
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
.debug-btn:hover {
  background: #4a4a4a;
  color: #fff;
}
.debug-btn.active {
  background: #b78e1c;
  color: #fff;
  border-color: #b78e1c;
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
.console-tabs {
  display: flex;
  gap: 4px;
}
.console-tabs button {
  padding: 4px 12px;
  background: none;
  border: none;
  color: #999;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  border-radius: 3px;
}
.console-tabs button:hover {
  color: #ccc;
}
.console-tabs button.active {
  background: #3c3c3c;
  color: #fff;
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
