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
      <div class="spacer" />
      <button
        v-if="!liveCompile"
        class="trans-btn"
        @click="$emit('trans')"
        title="Transpile now"
      >
        Trans
      </button>
      <label class="switch-widget" title="Toggle live transpile on edit">
        <span class="switch-label">Live</span>
        <span class="switch">
          <input
            type="checkbox"
            :checked="liveCompile"
            @change="$emit('toggleLive')"
          />
          <span class="slider"></span>
        </span>
      </label>
      <button
        class="icon-btn copy-icon-btn"
        @click="copyCode"
        :title="copied ? 'Copied!' : 'Copy transpiled code'"
      >
        <svg v-if="!copied" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
        <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      </button>
    </div>
    <div class="content">
      <CodePreview
        :code="transpiledCode"
        :language="activeTab"
        :highlight-lines="highlightLines"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { OutputTab } from '../types';
import CodePreview from './CodePreview.vue';

const props = defineProps<{
  activeTab: OutputTab;
  transpiledCode: string;
  liveCompile: boolean;
  highlightLines?: number[];
}>();

defineEmits<{
  tabChange: [tab: OutputTab];
  trans: [];
  toggleLive: [];
}>();

const tabs: { id: OutputTab; label: string }[] = [
  { id: 'rust', label: 'Rust' },
  { id: 'c', label: 'C' },
  { id: 'python', label: 'Python' },
  { id: 'javascript', label: 'JS' },
  { id: 'typescript', label: 'TS' },
];

const copied = ref(false);

async function copyCode() {
  try {
    await navigator.clipboard.writeText(props.transpiledCode);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch { /* ignore */ }
}
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
  align-items: center;
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
.spacer {
  flex: 1;
}
.trans-btn {
  background: #0e639c;
  color: #fff;
  border: none;
  border-radius: 4px;
  padding: 4px 14px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  margin-right: 8px;
  transition: background 0.15s;
}
.trans-btn:hover {
  background: #1177bb;
}

/* Switch widget */
.switch-widget {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  margin-right: 8px;
  user-select: none;
}
.switch-label {
  font-size: 12px;
  color: #ccc;
  font-weight: 500;
}
.switch {
  position: relative;
  display: inline-block;
  width: 32px;
  height: 18px;
}
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}
.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #555;
  transition: .2s;
  border-radius: 18px;
}
.slider:before {
  position: absolute;
  content: "";
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: .2s;
  border-radius: 50%;
}
.switch input:checked + .slider {
  background-color: #0e639c;
}
.switch input:checked + .slider:before {
  transform: translateX(14px);
}

/* Icon buttons */
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: #ccc;
  border: none;
  border-radius: 4px;
  padding: 4px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.icon-btn:hover {
  background: #3c3c3c;
  color: #fff;
}
.copy-icon-btn {
  margin-right: 4px;
}

.content {
  flex: 1;
  overflow: hidden;
}
</style>
