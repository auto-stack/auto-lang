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
}>();

defineEmits<{
  tabChange: [tab: OutputTab];
}>();

const tabs: { id: OutputTab; label: string }[] = [
  { id: 'console', label: 'Console' },
  { id: 'rust', label: 'Rust' },
  { id: 'c', label: 'C' },
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
}
.tab {
  background: transparent;
  color: #999;
  border: none;
  padding: 8px 16px;
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
.content {
  flex: 1;
  overflow: hidden;
}
</style>
