<template>
  <div class="output-panel">
    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab', { active: activeTab === tab.id }]"
        @click="$emit('tabChange', tab.id)"
      >
        {{ tab.label }}
      </button>
      <button
        :class="['tab', 'live-toggle', { active: liveCompile }]"
        @click="$emit('toggleLive')"
        title="Toggle live transpile on edit"
      >
        {{ liveCompile ? 'Live' : 'Manual' }}
      </button>
    </div>
    <div class="content">
      <ConsoleOutput
        v-if="activeTab === 'console'"
        :stdout="stdout"
        :stderr="stderr"
        :time-ms="timeMs"
      />
      <CodePreview
        v-else
        :code="transpiledCode"
        :language="activeTab"
        :highlight-lines="highlightLines"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { OutputTab } from '../types';
import ConsoleOutput from './ConsoleOutput.vue';
import CodePreview from './CodePreview.vue';

defineProps<{
  activeTab: OutputTab;
  stdout: string;
  stderr: string;
  timeMs: number;
  transpiledCode: string;
  liveCompile: boolean;
  highlightLines?: number[];
}>();

defineEmits<{
  tabChange: [tab: OutputTab];
  toggleLive: [];
}>();

const tabs: { id: OutputTab; label: string }[] = [
  { id: 'console', label: 'Console' },
  { id: 'rust', label: 'Rust' },
  { id: 'c', label: 'C' },
  { id: 'python', label: 'Python' },
  { id: 'javascript', label: 'JS' },
  { id: 'typescript', label: 'TS' },
];
</script>

<style scoped>
.output-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.tabs {
  display: flex;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
  padding: 0 4px;
  gap: 0;
}
.tab {
  background: transparent;
  color: #999;
  border: none;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 2px solid transparent;
}
.tab:hover {
  color: #ccc;
}
.tab.active {
  color: #fff;
  border-bottom-color: #007acc;
}
.live-toggle {
  margin-left: auto;
  font-size: 11px;
  padding: 8px 10px;
  border-radius: 3px;
}
.live-toggle.active {
  color: #4ec9b0;
  border-bottom-color: #4ec9b0;
}
.content {
  flex: 1;
  overflow: hidden;
}
</style>
