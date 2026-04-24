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
    @update:source="source = $event"
    @run="run"
    @trans="transpile(activeTab)"
    @tab-change="switchTab"
    @load-example="loadExample"
    @toggle-live="liveCompile = !liveCompile"
    @line-click="highlightSourceLine"
    @share="share"
  />
  <div class="toast" :class="{ visible: shareToast.visible }">
    {{ shareToast.message }}
  </div>
</template>

<script setup lang="ts">
import PlaygroundLayout from './components/PlaygroundLayout.vue';
import { usePlayground } from './composables/usePlayground';

const {
  source, stdout, stderr, resultCode, timeMs, isLoading,
  activeTab, transpiledCode, liveCompile,
  highlightedOutputLines, run, transpile, switchTab, loadExample, highlightSourceLine, share, shareToast,
} = usePlayground();
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
