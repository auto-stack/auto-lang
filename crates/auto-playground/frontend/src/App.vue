<template>
  <PlaygroundLayout
    :source="source"
    :is-loading="isLoading"
    :active-tab="activeTab"
    :stdout="stdout"
    :stderr="stderr"
    :result-code="resultCode"
    :time-ms="timeMs"
    :transpiled-code="transpiledCode"
    :live-compile="liveCompile"
    :highlight-lines="highlightedOutputLines"
    :on-run="run"
    :is-debugging="debug.isDebugging.value"
    :is-paused="debug.state.value?.status === 'paused'"
    :is-recording="debug.isRecording.value"
    :has-recording="!!debug.recording.value"
    :bytecode="activeBytecode"
    :debug-state="activeDebugState"
    :current-source-line="highlightedSourceLine"
    :highlighted-offsets="highlightedBytecodeOffsets"
    :breakpoints="breakpoints"
    :current-debug-line="activeDebugState?.line ?? null"
    :is-replay-mode="replay.isActive.value"
    :replay-current-index="replay.currentIndex.value"
    :replay-total-frames="replay.totalFrames.value"
    :is-replay-playing="replay.isPlaying.value"
    @update:source="source = $event"
    @run="run"
    @run-abt="runAbt"
    @run-code="runCode"
    @trans="transpile(activeTab)"
    @tab-change="switchTab"
    @load-example="loadExample"
    @toggle-live="liveCompile = !liveCompile"
    @line-click="highlightSourceLine"
    :on-highlight-line="highlightSourceLine"
    :on-clear-highlight="clearHighlight"
    @share="share"
    @toggle-debug="toggleDebug"
    @debug-command="onDebugCommand"
    @offset-click="onOffsetClick"
    @breakpoints-change="onBreakpointsChange"
    @toggle-record="toggleRecord"
    @export-recording="debug.exportRecording"
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
import { usePlayground } from './composables/usePlayground';
import { useDebugger } from './composables/useDebugger';
import { useReplayPlayer } from './composables/useReplayPlayer';
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import type { DebugRecording } from './types';

const {
  source, stdout, stderr, resultCode, timeMs, isLoading,
  activeTab, transpiledCode, liveCompile,
  highlightedOutputLines, highlightedSourceLine,
  run, runAbt, runCode, transpile, switchTab, loadExample, highlightSourceLine, clearHighlight, share, shareToast,
} = usePlayground();

const debug = useDebugger();
const replay = useReplayPlayer();
const breakpoints = ref<number[]>([]);

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

const highlightedBytecodeOffsets = computed(() => {
  if (!highlightedSourceLine.value) return undefined;
  if (replay.isActive.value) {
    return replay.lineToOffsets.value[highlightedSourceLine.value];
  }
  return debug.lineToOffsets.value[highlightedSourceLine.value];
});

// Sync debug finished state to main console so Run and Debug show the same result
watch(() => debug.state.value, (state) => {
  if (state?.status === 'finished') {
    stdout.value = state.stdout || '';
    resultCode.value = state.result || '';
    stderr.value = state.stderr || '';
  }
});

function toggleDebug() {
  if (debug.isDebugging.value) {
    debug.stop();
    if (debug.isRecording.value) {
      debug.stopRecording();
    }
    breakpoints.value = [];
  } else {
    // Stop replay if active
    replay.stop();
    debug.connect(source.value, breakpoints.value);
  }
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
  const line = replay.isActive.value
    ? replay.offsetToLine.value[offset]
    : debug.offsetToLine.value[offset];
  if (line) {
    highlightSourceLine(line);
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
    } catch (e) {
      alert('Failed to load replay file: ' + (e as Error).message);
    }
  };
  input.click();
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
      onDebugCommand('continue');
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
