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
    :bytecode="debug.bytecode.value"
    :debug-state="debug.state.value"
    :current-source-line="highlightedSourceLine"
    :highlighted-offsets="highlightedBytecodeOffsets"
    :breakpoints="breakpoints"
    :current-debug-line="debug.state.value?.line ?? null"
    @update:source="source = $event"
    @run="run"
    @trans="transpile(activeTab)"
    @tab-change="switchTab"
    @load-example="loadExample"
    @toggle-live="liveCompile = !liveCompile"
    @line-click="highlightSourceLine"
    @share="share"
    @toggle-debug="toggleDebug"
    @debug-command="onDebugCommand"
    @offset-click="onOffsetClick"
    @breakpoints-change="onBreakpointsChange"
  />
  <div class="toast" :class="{ visible: shareToast.visible }">
    {{ shareToast.message }}
  </div>
</template>

<script setup lang="ts">
import PlaygroundLayout from './components/PlaygroundLayout.vue';
import { usePlayground } from './composables/usePlayground';
import { useDebugger } from './composables/useDebugger';
import { computed, ref } from 'vue';

const {
  source, stdout, stderr, resultCode, timeMs, isLoading,
  activeTab, transpiledCode, liveCompile,
  highlightedOutputLines, highlightedSourceLine,
  run, transpile, switchTab, loadExample, highlightSourceLine, share, shareToast,
} = usePlayground();

const debug = useDebugger();
const breakpoints = ref<number[]>([]);

const highlightedBytecodeOffsets = computed(() => {
  if (!highlightedSourceLine.value) return undefined;
  return debug.lineToOffsets.value[highlightedSourceLine.value];
});

function toggleDebug() {
  if (debug.isDebugging.value) {
    debug.stop();
    breakpoints.value = [];
  } else {
    debug.connect(source.value);
  }
}

function onDebugCommand(cmd: 'continue' | 'step' | 'step_over' | 'step_out' | 'stop') {
  debug.sendCommand(cmd);
}

function onOffsetClick(offset: number) {
  const line = debug.offsetToLine.value[offset];
  if (line) {
    highlightSourceLine(line);
  }
}

function onBreakpointsChange(lines: number[]) {
  breakpoints.value = lines;
  debug.setBreakpoints(lines);
}
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
