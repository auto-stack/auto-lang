<template>
  <div class="debug-toolbar">
    <button
      v-for="btn in buttons"
      :key="btn.cmd"
      :disabled="!isPaused"
      @click="$emit('command', btn.cmd)"
      :title="btn.title"
    >
      <span class="icon">{{ btn.icon }}</span>
      <span class="label">{{ btn.label }}</span>
    </button>
    <button class="stop-btn" @click="$emit('command', 'stop')" title="Stop Debugging">
      <span class="icon">■</span>
      <span class="label">Stop</span>
    </button>

    <div class="toolbar-divider" />

    <!-- Recording controls -->
    <button
      :class="['record-btn', { recording: isRecording }]"
      @click="$emit('toggleRecord')"
      :title="isRecording ? 'Stop Recording' : 'Start Recording'"
    >
      <span class="icon">{{ isRecording ? '⏹' : '⏺' }}</span>
      <span class="label">{{ isRecording ? 'Recording' : 'Record' }}</span>
    </button>
    <button
      class="save-btn"
      @click="$emit('exportRecording')"
      :disabled="!hasRecording"
      title="Export Replay File"
    >
      <span class="icon">💾</span>
      <span class="label">Save</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { DebugCommand } from '../types';

defineProps<{
  isPaused: boolean;
  isRecording: boolean;
  hasRecording: boolean;
}>();

defineEmits<{
  command: [cmd: DebugCommand];
  toggleRecord: [];
  exportRecording: [];
}>();

const buttons: { cmd: DebugCommand; icon: string; label: string; title: string }[] = [
  { cmd: 'continue', icon: '▶', label: 'Continue', title: 'F5' },
  { cmd: 'step', icon: '↓', label: 'Step Into', title: 'F11' },
  { cmd: 'step_over', icon: '→', label: 'Step Over', title: 'F10' },
  { cmd: 'step_out', icon: '↑', label: 'Step Out', title: 'Shift+F11' },
];
</script>

<style scoped>
.debug-toolbar {
  display: flex;
  gap: 4px;
  padding: 4px 12px;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
  align-items: center;
  flex-shrink: 0;
}
.debug-toolbar button {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: #3c3c3c;
  border: 1px solid #555;
  border-radius: 3px;
  color: #ccc;
  cursor: pointer;
  font-size: 12px;
}
.debug-toolbar button:hover:not(:disabled) {
  background: #4a4a4a;
  color: #fff;
}
.debug-toolbar button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.stop-btn {
  color: #e51400 !important;
}
.stop-btn:hover {
  background: #4a1a1a !important;
}
.toolbar-divider {
  width: 1px;
  height: 20px;
  background: #555;
  margin: 0 4px;
}
.record-btn {
  color: #ccc;
}
.record-btn.recording {
  background: #5c1a1a !important;
  border-color: #e51400 !important;
  color: #ff6b6b !important;
}
.save-btn {
  color: #ccc;
}
</style>
