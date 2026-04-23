<template>
  <PlaygroundLayout
    :source="source"
    :is-loading="isLoading"
    :active-tab="activeTab"
    :stdout="stdout"
    :stderr="stderr"
    :time-ms="timeMs"
    :transpiled-code="transpiledCode"
    :live-compile="liveCompile"
    :highlight-lines="highlightedOutputLines"
    :on-run="run"
    @update:source="source = $event"
    @run="run"
    @tab-change="onTabChange"
    @load-example="loadExample"
    @toggle-live="liveCompile = !liveCompile"
    @line-click="highlightSourceLine"
  />
</template>

<script setup lang="ts">
import PlaygroundLayout from './components/PlaygroundLayout.vue';
import { usePlayground } from './composables/usePlayground';
import type { OutputTab } from './types';

const {
  source, stdout, stderr, timeMs, isLoading,
  activeTab, transpiledCode, liveCompile,
  highlightedOutputLines, run, transpile, loadExample, highlightSourceLine,
} = usePlayground();

function onTabChange(tab: OutputTab) {
  if (tab === 'console') {
    activeTab.value = tab;
  } else {
    activeTab.value = tab;
    transpile(tab);
  }
}
</script>
