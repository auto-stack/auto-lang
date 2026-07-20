<template>
  <div class="playground">
    <header class="toolbar">
      <div class="toolbar-left">
        <h1 class="title">Auto Playground</h1>
        <ExampleSelector @select="onLoadExample" />
      </div>
      <div class="toolbar-right">
        <button
          v-if="!isDebugging && !isReplayMode"
          class="toolbar-btn load-replay-btn"
          @click="$emit('loadReplay')"
          title="Load Replay File"
        >
          <span class="icon">📂</span>
          <span class="label">Load Replay</span>
        </button>
        <button class="toolbar-btn share-btn" @click="$emit('share')" title="Copy shareable link">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/>
            <polyline points="16 6 12 2 8 6"/>
            <line x1="12" y1="2" x2="12" y2="15"/>
          </svg>
          Share
        </button>
        <button
          class="toolbar-btn debug-btn"
          :class="{ active: isDebugging }"
          @click="props.onDebug"
          :disabled="isLoading || isReplayMode"
          title="Start Debugging"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2a10 10 0 0 1 10 10"/>
            <path d="M12 2a10 10 0 0 0-10 10"/>
            <path d="M12 12l4-4"/>
            <path d="M12 12l-4-4"/>
            <path d="M12 12l4 4"/>
            <path d="M12 12l-4 4"/>
          </svg>
          Debug
        </button>
        <button
          class="toolbar-btn run-btn"
          @click="props.onRun"
          :disabled="isLoading || isReplayMode"
        >
          {{ isLoading ? 'Running...' : 'Run (Ctrl+Enter)' }}
        </button>
        <div
          class="trans-split-btn"
          :class="{ disabled: isLoading || isReplayMode }"
          title="Transpile to target language"
        >
          <button
            class="trans-main"
            @click="props.onTrans"
            :disabled="isLoading || isReplayMode"
          >
            Trans
          </button>
          <div
            ref="transDropdownEl"
            class="trans-dropdown"
            :style="{ width: dropdownWidth }"
          >
            <select
              v-model="transTargetModel"
              class="trans-select"
              :disabled="isLoading || isDebugging || isReplayMode"
              @change="onTrans"
            >
              <option value="rust">Rust</option>
              <option value="c">C</option>
              <option value="python">Python</option>
              <option value="typescript">TypeScript</option>
              <option value="abt">ABT</option>
            </select>
            <span class="trans-current">{{ targetLabel }}</span>
            <span class="trans-arrow"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg></span>
          </div>
        </div>
      </div>
    </header>

    <!-- Debug toolbar -->
    <DebugToolbar
      v-if="isDebugging || hasRecording"
      :is-paused="isPaused"
      :is-recording="isRecording"
      :has-recording="hasRecording"
      @command="$emit('debugCommand', $event)"
      @toggle-record="$emit('toggleRecord')"
      @export-recording="$emit('exportRecording')"
    />

    <!-- Replay toolbar -->
    <ReplayToolbar
      v-if="isReplayMode"
      :is-playing="isReplayPlaying"
      :current-index="replayCurrentIndex"
      :total-frames="replayTotalFrames"
      @play="$emit('replayPlay')"
      @pause="$emit('replayPause')"
      @step-forward="$emit('replayStepForward')"
      @step-backward="$emit('replayStepBackward')"
      @seek="$emit('replaySeek', $event)"
    />

    <div class="workspace">
      <div class="main-row">
        <div class="editor-pane" :class="{ 'with-preview': mode !== 'editor' }">
          <div class="pane-header">
            <span v-if="isReplayMode">Replay</span>
            <span v-else>Auto <span v-if="activeFile" class="active-file-name">· {{ activeFile }}</span></span>
          </div>
          <div class="pane-body">
            <FileTree
              v-if="showSourceFileTree"
              :files="projectFiles!"
              :selected="activeFile || ''"
              :mapped-files="mappedSourceFilesArray"
              @select="$emit('selectFile', $event)"
            />
            <CodeEditor
              :model-value="source"
              @update:model-value="$emit('update:source', $event)"
              :on-run="onRun"
              :is-debugging="isDebugging || isReplayMode"
              :breakpoints="breakpoints"
              :current-debug-line="currentDebugLine"
              :highlighted-source-line="currentSourceLine"
              :read-only="isReplayMode"
              @line-click="$emit('lineClick', $event)"
              @breakpoints-change="$emit('breakpointsChange', $event)"
              @hover-line="props.onHighlightLine?.($event)"
              @hover-line-leave="props.onClearHighlight?.()"
            />
          </div>
        </div>
        <div v-if="mode !== 'editor'" class="preview-pane">
          <div class="pane-header">
            <span>{{ previewTitle }}</span>
            <button
              v-if="canRunCode"
              class="run-code-btn"
              :disabled="isLoading || isReplayMode"
              @click="onRunCodeClick"
            >
              Run {{ targetLabel }}
            </button>
          </div>
          <div class="pane-body" :class="{ 'with-file-tree': showFileTree }">
            <BytecodePanel
              v-if="mode === 'run' || mode === 'debug' || mode === 'replay'"
              :bytecode="effectiveBytecode"
              :current-ip="debugState?.ip"
              :highlighted-offsets="highlightedOffsets"
              @offset-click="$emit('offsetClick', $event)"
            />
            <template v-else-if="mode === 'trans'">
              <FileTree
                v-if="showFileTree"
                :files="transFiles || []"
                :selected="selectedTransFile || ''"
                @select="onSelectTransFile"
              />
              <CodePreview
                :code="transpiledCode"
                :language="previewLanguage"
                :highlight-lines="highlightLines"
                @line-click="onOutputLineClick"
              />
            </template>
          </div>
        </div>
      </div>

      <div v-if="showOutputPanel" class="output-pane">
        <div class="pane-header">
          <span>{{ outputTitle }}</span>
        </div>
        <div class="output-body">
          <ConsoleOutput
            class="console-main"
            :stdout="stdout"
            :stderr="stderr"
            :result="resultCode"
            :time-ms="timeMs"
          />
          <DebugAuxPanel
            v-if="(isDebugging || isReplayMode) && debugState"
            :state="debugState"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import type { OutputTab, BytecodeLine, DebugState, TransFile, ProjectFile } from '../types';
import CodeEditor from './CodeEditor.vue';
import CodePreview from './CodePreview.vue';
import BytecodePanel from './BytecodePanel.vue';
import ConsoleOutput from './ConsoleOutput.vue';
import ExampleSelector from './ExampleSelector.vue';
import DebugToolbar from './DebugToolbar.vue';
import ReplayToolbar from './ReplayToolbar.vue';
import DebugAuxPanel from './DebugAuxPanel.vue';
import FileTree from './FileTree.vue';

type PlaygroundMode = 'editor' | 'run' | 'trans' | 'debug' | 'replay';

const props = defineProps<{
  source: string;
  isLoading: boolean;
  mode: PlaygroundMode;
  transTarget: OutputTab;
  stdout: string;
  stderr: string;
  resultCode: string;
  timeMs: number;
  transpiledCode: string;
  transFiles?: TransFile[];
  selectedTransFile?: string;
  highlightLines?: number[];
  projectFiles?: ProjectFile[];
  activeFile?: string;
  mappedSourceFiles?: Set<string>;
  onRun: () => void;
  onTrans: () => void;
  onRunCode?: (language: string) => void;
  onDebug: () => void;
  onSelectTransFile?: (target: string, path: string) => void;
  onOutputLineClick?: (outputFile: string, outputLine: number) => void;
  // Debug props
  isDebugging?: boolean;
  isPaused?: boolean;
  isRecording?: boolean;
  hasRecording?: boolean;
  bytecode?: BytecodeLine[];
  debugState?: DebugState | null;
  currentSourceLine?: number | null;
  highlightedOffsets?: number[];
  breakpoints?: number[];
  currentDebugLine?: number | null;
  // Replay props
  isReplayMode?: boolean;
  replayCurrentIndex?: number;
  replayTotalFrames?: number;
  isReplayPlaying?: boolean;
  // Highlight callbacks
  onHighlightLine?: (line: number) => void;
  onClearHighlight?: () => void;
}>();

const emit = defineEmits<{
  'update:source': [value: string];
  'update:transTarget': [value: OutputTab];
  loadExample: [payload: { source: string; project_dir?: string; files?: ProjectFile[] }];
  selectFile: [path: string];
  share: [];
  // Debug events
  debugCommand: [cmd: 'continue' | 'step' | 'step_over' | 'step_out' | 'stop'];
  toggleRecord: [];
  exportRecording: [];
  lineClick: [line: number];
  offsetClick: [offset: number];
  breakpointsChange: [lines: number[]];
  // Replay events
  loadReplay: [];
  replayPlay: [];
  replayPause: [];
  replayStepForward: [];
  replayStepBackward: [];
  replaySeek: [index: number];
}>();

const transTargetModel = computed({
  get: () => props.transTarget,
  set: (v) => emit('update:transTarget', v),
});

const targetLabel = computed(() => {
  const labels: Record<OutputTab, string> = {
    rust: 'Rust',
    c: 'C',
    python: 'Python',
    typescript: 'TypeScript',
    abt: 'ABT',
    bytecode: 'Bytecode',
  };
  return labels[props.transTarget] ?? props.transTarget;
});

const transDropdownEl = ref<HTMLElement | null>(null);
const dropdownWidth = ref('auto');

onMounted(async () => {
  await document.fonts?.ready;
  const el = transDropdownEl.value;
  if (!el) return;
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  const style = getComputedStyle(el);
  ctx.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
  const labels = ['Rust', 'C', 'Python', 'TypeScript', 'ABT'];
  let maxTextWidth = 0;
  for (const label of labels) {
    maxTextWidth = Math.max(maxTextWidth, ctx.measureText(label).width);
  }
  const arrowWidth = 12;
  const padH = 10;
  const textArrowGap = 4;
  dropdownWidth.value = `${Math.ceil(maxTextWidth + arrowWidth + padH * 2 + textArrowGap)}px`;
});

const codeRunActive = ref(false);

watch(() => props.mode, () => {
  codeRunActive.value = false;
});

const canRunCode = computed(() => {
  const lang = props.transTarget;
  return props.mode === 'trans' && (lang === 'python' || lang === 'typescript');
});

async function onRunCodeClick() {
  const lang = props.transTarget;
  if (!lang || !props.onRunCode) return;
  codeRunActive.value = true;
  await props.onRunCode(lang);
}

const previewTitle = computed(() => {
  if (props.mode === 'run' || props.mode === 'debug' || props.mode === 'replay') return 'Bytecode';
  if (props.mode === 'trans') return targetLabel.value;
  return '';
});

const previewLanguage = computed(() => {
  if (props.mode === 'trans') return props.transTarget;
  return undefined;
});

const showFileTree = computed(() => props.mode === 'trans' && (props.transFiles?.length ?? 0) > 1);

const showSourceFileTree = computed(() => (props.projectFiles?.length ?? 0) > 1);

const mappedSourceFilesArray = computed(() =>
  props.mappedSourceFiles ? Array.from(props.mappedSourceFiles) : []
);

function onOutputLineClick(line: number) {
  props.onOutputLineClick?.(props.selectedTransFile ?? '', line);
}

const effectiveBytecode = computed(() => {
  if (props.mode === 'run') return props.bytecode ?? [];
  return props.bytecode ?? [];
});

function onSelectTransFile(path: string) {
  props.onSelectTransFile?.(props.transTarget, path);
}

const showOutputPanel = computed(() =>
  props.mode === 'run' || props.mode === 'debug' || props.mode === 'replay' || codeRunActive.value
);

const outputTitle = computed(() => {
  if (props.mode === 'run' || codeRunActive.value) return 'Output';
  if (props.mode === 'debug' || props.mode === 'replay') return 'Debug Output';
  return '';
});

function onTrans() {
  props.onTrans();
}

function onLoadExample(payload: { source: string; project_dir?: string; files?: ProjectFile[] }) {
  emit('loadExample', payload);
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
.toolbar-btn {
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
  transition: background 0.15s, color 0.15s;
}
.toolbar-btn:hover:not(:disabled) {
  background: #4a4a4a;
  color: #fff;
}
.toolbar-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.debug-btn.active {
  background: #b78e1c;
  color: #fff;
  border-color: #b78e1c;
}
.run-btn {
  background: #0e639c;
  color: #fff;
  border-color: #0e639c;
}
.run-btn:hover:not(:disabled) {
  background: #1177bb;
}
.trans-split-btn {
  display: inline-flex;
  align-items: stretch;
  background: #238636;
  color: #fff;
  border: 1px solid #238636;
  border-radius: 4px;
  overflow: hidden;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
}
.trans-split-btn:hover:not(.disabled) {
  background: #2ea043;
  border-color: #2ea043;
}
.trans-split-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.trans-main {
  background: transparent;
  color: inherit;
  border: none;
  padding: 6px 12px;
  font-size: inherit;
  font-weight: inherit;
  cursor: pointer;
  border-right: 1px solid rgba(255, 255, 255, 0.25);
}
.trans-main:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}
.trans-main:disabled {
  cursor: not-allowed;
}
.trans-dropdown {
  position: relative;
  display: inline-flex;
  align-items: center;
  min-width: 72px;
  padding: 0 10px;
}
.trans-dropdown:hover {
  background: rgba(255, 255, 255, 0.1);
}
.trans-select {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
  border: none;
  outline: none;
}
.trans-select:disabled {
  cursor: not-allowed;
}
.trans-current {
  pointer-events: none;
  flex: 1;
  font-size: inherit;
  font-weight: inherit;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  text-align: right;
  padding-right: 20px;
}
.trans-arrow {
  pointer-events: none;
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  display: inline-flex;
  align-items: center;
  color: #fff;
}
.trans-select option {
  background: #1e1e1e;
  color: #d4d4d4;
}
.workspace {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-row {
  flex: 2;
  display: flex;
  overflow: hidden;
}
.editor-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.editor-pane.with-preview {
  border-right: 1px solid #444;
}
.preview-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.output-pane {
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
  text-transform: capitalize;
}
.run-code-btn {
  background: #0e639c;
  color: #fff;
  border: 1px solid #0e639c;
  border-radius: 4px;
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}
.run-code-btn:hover:not(:disabled) {
  background: #1177bb;
}
.run-code-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.pane-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: row;
}

.active-file-name {
  text-transform: none;
  color: #89b4fa;
}

.output-body {
  flex: 1;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}
.console-main {
  flex: 1;
  overflow: auto;
}

@media (max-width: 768px) {
  .main-row {
    flex-direction: column;
  }
  .editor-pane.with-preview {
    border-right: none;
    border-bottom: 1px solid #444;
  }
}
</style>
