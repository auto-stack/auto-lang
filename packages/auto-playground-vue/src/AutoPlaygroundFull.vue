<template>
  <PlaygroundLayout
    :source="source"
    :is-loading="isLoading"
    :mode="mode"
    :trans-target="transTarget"
    @update:trans-target="transTarget = $event"
    :stdout="stdout"
    :stderr="stderr"
    :result-code="resultCode"
    :time-ms="timeMs"
    :transpiled-code="transpiledCode"
    :trans-files="transFiles"
    :selected-trans-file="selectedTransFile"
    :project-files="projectFiles"
    :active-file="activeFile"
    :mapped-source-files="mappedSourceFiles"
    :highlight-lines="highlightedOutputLines"
    :on-run="onRun"
    :on-trans="onTrans"
    :on-run-code="onRunCode"
    :on-debug="onDebug"
    :on-select-trans-file="selectTransFile"
    :on-output-line-click="highlightOutputLine"
    :is-debugging="debug.isDebugging.value"
    :is-paused="debug.state.value?.status === 'paused'"
    :is-recording="debug.isRecording.value"
    :has-recording="!!debug.recording.value"
    :bytecode="layoutBytecode"
    :bytecode-meta="mode === 'run' ? runBytecodeMeta : activeMeta"
    :debug-state="activeDebugState"
    :current-source-line="hoveredSourceLine"
    :highlighted-offsets="hoveredOffsets"
    :selected-offsets="selectedOffsets"
    :selected-source-line="highlightedSourceLine"
    :breakpoints="breakpoints"
    :current-debug-line="activeDebugState?.line ?? null"
    :is-replay-mode="replay.isActive.value"
    :replay-current-index="replay.currentIndex.value"
    :replay-total-frames="replay.totalFrames.value"
    :is-replay-playing="replay.isPlaying.value"
    @update:source="source = $event"
    @load-example="onLoadExample"
    @select-file="selectFile"
    @share="share"
    @debug-command="onDebugCommand"
    @toggle-record="toggleRecord"
    @export-recording="debug.exportRecording"
    @line-click="selectSourceLine"
    :on-highlight-line="onHoverLine"
    :on-clear-highlight="onHoverLeave"
    @offset-click="onOffsetClick"
    @breakpoints-change="onBreakpointsChange"
    @load-replay="onLoadReplay"
    @replay-play="replay.play"
    @replay-pause="replay.pause"
    @replay-step-forward="replay.stepForward"
    @replay-step-backward="replay.stepBackward"
    @replay-seek="replay.seek"
  />
  <div class="toast" :class="{ visible: shareToast.visible }">
    {{ shareToast.message }}
  </div>
</template>

<script setup lang="ts">
import PlaygroundLayout from './components/PlaygroundLayout.vue';
import { usePlaygroundFull } from './composables/usePlaygroundFull';
import { useDebugger } from './composables/useDebugger';
import { useReplayPlayer } from './composables/useReplayPlayer';
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import type { DebugRecording, OutputTab, ProjectFile } from './types';

type PlaygroundMode = 'editor' | 'run' | 'trans' | 'debug' | 'replay';

const {
  source, stdout, stderr, resultCode, timeMs, bytecode: runBytecode, bytecodeMeta: runBytecodeMeta, isLoading,
  activeTab, transpiledCode, transFiles, selectedTransFile,
  projectFiles, activeFile,
  highlightedOutputLines, highlightedSourceLine, mappedSourceFiles,
  run, transpile, runCode, selectTransFile, selectFile, loadExample, highlightOutputLine, share, shareToast,
} = usePlaygroundFull();

const debug = useDebugger();
const replay = useReplayPlayer();
const breakpoints = ref<number[]>([]);

const mode = ref<PlaygroundMode>('editor');
const transTarget = ref<OutputTab>('rust');

// When in replay mode, use replay state; otherwise use debug state
const activeDebugState = computed(() => {
  if (replay.isActive.value) {
    return replay.currentState.value;
  }
  return debug.state.value;
});

const activeBytecode = computed(() => {
  if (replay.isActive.value) {
    return replay.bytecode.value;
  }
  return debug.bytecode.value;
});

const activeMeta = computed(() => {
  if (replay.isActive.value) {
    return replay.meta.value;
  }
  return debug.meta.value;
});

const layoutBytecode = computed(() => {
  if (mode.value === 'run') {
    return runBytecode.value;
  }
  return activeBytecode.value;
});

// Line↔offset maps derived from whatever bytecode is on screen, so source
// clicks highlight bytecode in Run mode too (same mechanism Debug used).
const bytecodeLineToOffsets = computed(() => {
  const map: Record<number, number[]> = {};
  for (const line of layoutBytecode.value) {
    if (line.line !== undefined) {
      if (!map[line.line]) map[line.line] = [];
      map[line.line].push(line.offset);
    }
  }
  return map;
});

const bytecodeOffsetToLine = computed(() => {
  const map: Record<number, number> = {};
  for (const line of layoutBytecode.value) {
    if (line.line !== undefined) {
      map[line.offset] = line.line;
    }
  }
  return map;
});

// Two highlight layers: pinned selection (clicks) vs transient hover.
// `highlightedSourceLine` (from usePlaygroundFull) carries the selection;
// hover is component-local so it never fights the pinned line.
const hoveredSourceLine = ref<number | null>(null);
const selectedOffsets = computed(() =>
  highlightedSourceLine.value ? bytecodeLineToOffsets.value[highlightedSourceLine.value] : undefined);
const hoveredOffsets = computed(() =>
  hoveredSourceLine.value ? bytecodeLineToOffsets.value[hoveredSourceLine.value] : undefined);

function selectSourceLine(line: number) {
  highlightedSourceLine.value = line;
}
function onHoverLine(line: number) {
  hoveredSourceLine.value = line;
}
function onHoverLeave() {
  hoveredSourceLine.value = null;
}

// Sync debug finished state to main console so Run and Debug show the same result
watch(() => debug.state.value, (state) => {
  if (state?.status === 'finished') {
    stdout.value = state.stdout || '';
    resultCode.value = state.result || '';
    stderr.value = state.stderr || '';
  }
});

// Reset UI when debug session ends, but keep the result visible
// when the program ran to completion
watch(() => debug.isDebugging.value, (isDebugging) => {
  if (!isDebugging && mode.value === 'debug' && debug.state.value?.status !== 'finished') {
    mode.value = 'editor';
  }
});

// Reset UI when replay session ends
watch(() => replay.isActive.value, (isActive) => {
  if (!isActive && mode.value === 'replay') {
    mode.value = 'editor';
  }
});

async function onRun() {
  mode.value = 'run';
  stdout.value = '';
  stderr.value = '';
  resultCode.value = '';
  await run();
}

async function onTrans() {
  mode.value = 'trans';
  await transpile(transTarget.value);
  activeTab.value = transTarget.value;
}

async function onRunCode(language: string) {
  await runCode(language);
}

function onDebug() {
  if (debug.isDebugging.value) return;
  mode.value = 'debug';
  // Stop replay if active
  replay.stop();
  debug.connect(source.value, breakpoints.value);
}

function toggleRecord() {
  if (debug.isRecording.value) {
    debug.stopRecording();
  } else {
    debug.startRecording(source.value, breakpoints.value);
  }
}

function onDebugCommand(cmd: 'continue' | 'step' | 'step_over' | 'step_out' | 'stop') {
  debug.sendCommand(cmd);
}

function onOffsetClick(offset: number) {
  const line = bytecodeOffsetToLine.value[offset];
  if (line) {
    selectSourceLine(line);
  }
}

function onBreakpointsChange(lines: number[]) {
  breakpoints.value = lines;
  debug.setBreakpoints(lines);
}

async function onLoadReplay() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.autoreplay,.json';
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const data = JSON.parse(text) as DebugRecording;
      // Stop any active debug/replay
      debug.stop();
      replay.load(data);
      mode.value = 'replay';
    } catch (e) {
      alert('Failed to load replay file: ' + (e as Error).message);
    }
  };
  input.click();
}

function onLoadExample(payload: { source: string; project_dir?: string; files?: ProjectFile[] }) {
  loadExample(payload);
  mode.value = 'editor';
}

function onKeyDown(e: KeyboardEvent) {
  if (replay.isActive.value) {
    switch (e.key) {
      case 'ArrowRight':
        e.preventDefault();
        replay.stepForward();
        break;
      case 'ArrowLeft':
        e.preventDefault();
        replay.stepBackward();
        break;
      case ' ':
        e.preventDefault();
        replay.isPlaying.value ? replay.pause() : replay.play();
        break;
    }
    return;
  }
  if (!debug.isDebugging.value) return;
  switch (e.key) {
    case 'F5':
      e.preventDefault();
      onDebugCommand(e.shiftKey ? 'stop' : 'continue');
      break;
    case 'F10':
      e.preventDefault();
      onDebugCommand('step_over');
      break;
    case 'F11':
      e.preventDefault();
      onDebugCommand(e.shiftKey ? 'step_out' : 'step');
      break;
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown);
  // Test hook: expose replay loader for e2e tests
  (window as any).__loadReplayForTest__ = (data: any) => {
    replay.load(data);
    mode.value = 'replay';
  };
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
});
</script>

<style>
.toast {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%) translateY(-120%);
  background: #252526;
  color: #fff;
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #444;
  z-index: 1000;
  opacity: 0;
  transition: all 0.3s ease;
  pointer-events: none;
}
.toast.visible {
  transform: translateX(-50%) translateY(0);
  opacity: 1;
}
</style>
